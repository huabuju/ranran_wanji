import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "DeviceOverview",
    component: () => import("@/views/overview/index.vue"),
    meta: { title: "仪表概览", icon: "overview", color: "#3b82f6" },
  },
  {
    path: "/device-control",
    name: "DeviceControl",
    component: () => import("@/views/device-control/index.vue"),
    meta: { title: "设备控制", icon: "controller", color: "#f97316" },
  },
  {
    path: "/scrcpy",
    name: "ScreenControl",
    component: () => import("@/views/screen-control/index.vue"),
    meta: { title: "投屏控制", icon: "scrcpy", color: "#a855f7" },
  },
  {
    path: "/file-manager",
    name: "FileManager",
    component: () => import("@/views/file-manager/index.vue"),
    meta: { title: "文件管理", icon: "folder", color: "#f59e0b" },
  },
  {
    path: "/apps",
    name: "AppsManagement",
    component: () => import("@/views/apps/index.vue"),
    meta: { title: "应用管理", icon: "package", color: "#ef4444" },
  },
  {
    path: "/sys-props",
    name: "SysProps",
    component: () => import("@/views/sys-props/index.vue"),
    meta: { title: "系统属性", icon: "settings", color: "#3b82f6" },
  },
  {
    path: "/link-dumper",
    name: "LinkDumper",
    component: () => import("@/views/link-dumper/index.vue"),
    meta: { title: "镜像提取", icon: "package", color: "#6366f1" },
  },
  {
    path: "/rom",
    name: "RomDownload",
    component: () => import("@/views/rom-download/index.vue"),
    meta: { title: "ROM 下载", icon: "download", color: "#06b6d4" },
  },
  {
    path: "/github-apk",
    name: "GitHubApk",
    component: () => import("@/views/github-apk/index.vue"),
    meta: { title: "GitHub APK", icon: "github", color: "#94a3b8" },
  },
  {
    path: "/boot-patch",
    name: "BootPatch",
    component: () => import("@/views/boot-patch/index.vue"),
    meta: { title: "Boot 修补", icon: "bolt", color: "#0ea5e9" },
  },
  {
    path: "/partition",
    name: "PartitionManagement",
    component: () => import("@/views/partition/index.vue"),
    meta: { title: "分区管理", icon: "partition", color: "#10b981" },
  },
  {
    path: "/custom-commands",
    name: "CustomCommands",
    component: () => import("@/views/custom-commands/index.vue"),
    meta: { title: "自定义指令", icon: "terminal", color: "#db2777" },
  },
  {
    path: "/downloader",
    name: "Downloader",
    component: () => import("@/views/downloader/index.vue"),
    meta: { title: "下载器", icon: "download", color: "#3b82f6" },
  },
  {
    path: "/about",
    name: "About",
    component: () => import("@/views/about/index.vue"),
    meta: { title: "关于我们", icon: "about", color: "#64748b" },
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
