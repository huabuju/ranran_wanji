import android.content.pm.ApplicationInfo;
import android.os.Looper;

import java.lang.reflect.Method;
import java.util.List;

/**
 * AppLabelHelper - 通过 app_process 以系统身份获取所有应用名与包名的映射
 *
 * 运行方式:
 *   adb push AppLabelHelper.dex /data/local/tmp/
 *   adb shell app_process -Djava.class.path=/data/local/tmp/AppLabelHelper.dex \
 *             /data/local/tmp Main [-a]
 *
 * 参数:
 *   -3  仅输出第三方用户应用（默认）
 *   -s  仅输出系统应用
 *   -a  输出全部应用
 *
 * 输出格式 (UTF-8):
 *   [{"package_name":"","label":"","uid":0,"version_name":"","source_dir":"/data/app/..."},...]
 */
public class Main {
    public static void main(String[] args) throws Exception {
        // 初始化主线程 Looper（app_process 环境必须）
        Looper.prepareMainLooper();

        // 通过反射调用 ActivityThread.systemMain().getSystemContext() 获取 Context
        // 避免编译时依赖完整的 android.jar 导致找不到 ActivityThread 类
        Class<?> activityThreadClass = Class.forName("android.app.ActivityThread");
        Method systemMainMethod = activityThreadClass.getDeclaredMethod("systemMain");
        systemMainMethod.setAccessible(true);
        Object activityThread = systemMainMethod.invoke(null);

        Method getSystemContextMethod = activityThreadClass.getDeclaredMethod("getSystemContext");
        getSystemContextMethod.setAccessible(true);
        Object contextObj = getSystemContextMethod.invoke(activityThread);

        Method getPackageManagerMethod = contextObj.getClass().getMethod("getPackageManager");
        getPackageManagerMethod.setAccessible(true);
        Object pmObj = getPackageManagerMethod.invoke(contextObj);

        Method getInstalledApplicationsMethod = pmObj.getClass().getMethod("getInstalledApplications", int.class);
        getInstalledApplicationsMethod.setAccessible(true);

        Method getPackageInfoMethod = pmObj.getClass().getMethod("getPackageInfo", String.class, int.class);
        getPackageInfoMethod.setAccessible(true);
        Class<?> packInfoClass = Class.forName("android.content.pm.PackageInfo");
        java.lang.reflect.Field versionNameField = packInfoClass.getField("versionName");

        Class<?> appInfoClassRef = Class.forName("android.content.pm.ApplicationInfo");
        Method getApplicationLabelMethod = pmObj.getClass().getMethod("getApplicationLabel", appInfoClassRef);
        getApplicationLabelMethod.setAccessible(true);

        // 用于获取 APK 安装路径 (sourceDir)
        java.lang.reflect.Field sourceDirField = appInfoClassRef.getField("sourceDir");

        // 解析过滤参数
        String filter = "-3"; // 默认仅第三方应用
        if (args.length > 0) {
            filter = args[0];
        }

        // GET_DISABLED_COMPONENTS = 0x200 | MATCH_UNINSTALLED_PACKAGES = 0x2000
        int flags = 0x2200;
        
        @SuppressWarnings("unchecked")
        List<Object> apps = (List<Object>) getInstalledApplicationsMethod.invoke(pmObj, flags);

        // 通过反射获取 ApplicationInfo 的字段
        Class<?> appInfoClass = Class.forName("android.content.pm.ApplicationInfo");
        java.lang.reflect.Field flagsField = appInfoClass.getField("flags");
        java.lang.reflect.Field packageNameField = appInfoClass.getField("packageName");
        java.lang.reflect.Field uidField = appInfoClass.getField("uid");
        int FLAG_SYSTEM = appInfoClass.getField("FLAG_SYSTEM").getInt(null);
        java.lang.reflect.Field sourceDirFieldLocal = appInfoClass.getField("sourceDir");

        StringBuilder sb = new StringBuilder();
        sb.append("[");
        boolean first = true;

        for (Object app : apps) {
            // 根据 flags 过滤
            int appFlags = flagsField.getInt(app);
            boolean isSystem = (appFlags & FLAG_SYSTEM) != 0;
            if ("-3".equals(filter) && isSystem) continue;
            if ("-s".equals(filter) && !isSystem) continue;

            String packageName = (String) packageNameField.get(app);
            int uid = uidField.getInt(app);

            String labelStr = packageName;
            String versionName = "Unknown";
            String sourceDir = "";
            try {
                CharSequence label = (CharSequence) getApplicationLabelMethod.invoke(pmObj, app);
                if (label != null) labelStr = label.toString();

                try {
                    Object packageInfoObj = getPackageInfoMethod.invoke(pmObj, packageName, 0); 
                    if (packageInfoObj != null) {
                        versionName = (String) versionNameField.get(packageInfoObj);
                        if (versionName == null) versionName = "Unknown";
                    }
                } catch (Exception e) {
                    // ignore
                }

                // 获取 APK 安装路径
                try {
                    String sd = (String) sourceDirFieldLocal.get(app);
                    if (sd != null) sourceDir = sd;
                } catch (Exception e) {
                    // ignore
                }
            } catch (Exception e) {
                // ignore
            }

            if (!first) {
                sb.append(",");
            }
            first = false;

            sb.append("{");
            sb.append("\"package_name\":\"").append(escapeJson(packageName)).append("\",");
            sb.append("\"label\":\"").append(escapeJson(labelStr)).append("\",");
            sb.append("\"uid\":").append(uid).append(",");
            sb.append("\"version_name\":\"").append(escapeJson(versionName)).append("\",");
            sb.append("\"source_dir\":\"").append(escapeJson(sourceDir)).append("\"");
            sb.append("}");
        }
        sb.append("]");
        System.out.println(sb.toString());
    }

    private static String escapeJson(String str) {
        if (str == null) return "";
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < str.length(); i++) {
            char ch = str.charAt(i);
            switch (ch) {
                case '"': sb.append("\\\""); break;
                case '\\': sb.append("\\\\"); break;
                case '\b': sb.append("\\b"); break;
                case '\f': sb.append("\\f"); break;
                case '\n': sb.append("\\n"); break;
                case '\r': sb.append("\\r"); break;
                case '\t': sb.append("\\t"); break;
                default:
                    if (ch < 32) {
                        String ss = Integer.toHexString(ch);
                        sb.append("\\u");
                        for (int k = 0; k < 4 - ss.length(); k++) sb.append('0');
                        sb.append(ss.toUpperCase());
                    } else {
                        sb.append(ch);
                    }
            }
        }
        return sb.toString();
    }
}
