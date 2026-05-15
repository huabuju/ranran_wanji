import { invoke } from '@tauri-apps/api/core';
import { useDeviceStore } from '@/utils/deviceStore';

// =====================================================
// 内部工具：获取当前选中的设备 serial
// =====================================================
function getSerial() {
  const { selectedSerial } = useDeviceStore();
  return selectedSerial.value || null;
}

/**
 * 获取设备基本信息
 * @returns {Promise<Object>}
 */
export async function fetchDeviceInfo() {
  return await invoke('get_device_info', { serial: getSerial() });
}

/**
 * 获取已连接设备列表（枚举所有设备，无需 serial）
 * @returns {Promise<{
 *   devices: Array<{serial: string, state: string, source: string}>,
 *   adb_report: {raw_output: string, devices: Array},
 *   fastboot_report: {raw_output: string, devices: Array},
 *   duration_ms: number
 * }>}
 */
export async function fetchConnectedDevices() {
  return await invoke('get_connected_devices');
}

/**
 * 获取带诊断信息的设备扫描快照
 * @returns {Promise<{
 *   devices: Array<{serial: string, state: string, source: string}>,
 *   adb_report: {raw_output: string, devices: Array},
 *   fastboot_report: {raw_output: string, devices: Array},
 *   duration_ms: number
 * }>}
 */

/**
 * 获取设备分区信息
 * @returns {Promise<Array>}
 */
export async function fetchPartitionInfo() {
  return await invoke('get_partition_info', { serial: getSerial() });
}

/**
 * 获取设备资源监控数据 (CPU/内存/存储/电池等)
 * @returns {Promise<Object>}
 */
export async function fetchDeviceResources() {
  return await invoke('get_device_resources', { serial: getSerial() });
}

/**
 * 唤起宿主的 CMD / PowerShell 窗口
 * @returns {Promise<void>}
 */
export async function openPlatformToolsCmd() {
  return await invoke('open_platform_tools_cmd');
}

/**
 * 唤起宿主的设备管理器
 * @returns {Promise<void>}
 */
export async function openDeviceManager() {
  return await invoke('open_device_manager');
}



/**
 * 重启应用
 * @returns {Promise<void>}
 */
export async function restartApplication() {
  return await invoke('restart_app');
}

/**
 * 发送 ADB Keyevent
 * @param {string} keycode - 按键代码
 * @returns {Promise<void>}
 */
export async function sendKeyevent(keycode) {
  return await invoke('adb_send_keyevent', { serial: getSerial(), keycode: String(keycode) });
}

/**
 * 获取设备应用状态 (系统应用, 用户应用, 总量)
 * @returns {Promise<Object>}
 */
export async function fetchAppStatus() {
  return await invoke('get_app_status', { serial: getSerial() });
}

/**
 * 执行通用 ADB Shell 命令
 * @param {string[]} args - 命令参数数组
 * @returns {Promise<string>}
 */
export async function runAdbShell(args) {
  return await invoke('run_adb_shell', { serial: getSerial(), args });
}

/**
 * 执行通用 ADB 命令 (非 Shell)
 * @param {string[]} args - 命令参数数组
 * @param {{ useCurrentSerial?: boolean }} options - 是否附带当前选中设备 serial
 * @returns {Promise<string>}
 */
export async function runAdb(args, options = {}) {
  const { useCurrentSerial = true } = options;
  return await invoke('run_adb', { serial: useCurrentSerial ? getSerial() : null, args });
}

/**
 * 执行设备重启相关功能 (自动检测模式)
 * @param {string} target - 重启目标 (system, bootloader, recovery, poweroff)
 * @returns {Promise<string>}
 */
export async function deviceReboot(target) {
  return await invoke('device_reboot', { serial: getSerial(), target });
}

/**
 * 获取设备状态 (adb/fastboot/none等)
 * @returns {Promise<string>}
 */
export async function fetchDeviceState() {
  return await invoke('get_device_state', { serial: getSerial() });
}

/**
 * 扫描局域网内的无线 ADB 设备
 * @returns {Promise<Array<{instance_name: string, service_type: string, ip: string, port: string}>>}
 */
export async function scanAdbDevices() {
  return await invoke('scan_adb_devices');
}

/**
 * 执行 ADB 配对
 * @param {string} addr - IP:Port
 * @param {string} code - 6 位配对码
 * @returns {Promise<string>}
 */
export async function adbPair(addr, code) {
  return await invoke('adb_pair', { addr, code });
}

/**
 * 执行 Fastboot 命令
 * @param {string[]} args - 命令参数数组
 * @returns {Promise<string>}
 */
export async function runFastboot(args) {
  return await invoke('run_fastboot', { serial: getSerial(), args });
}

/**
 * 检查宿主机文件是否存在
 * @param {string} path - 路径
 * @returns {Promise<boolean>}
 */
export async function checkFileExists(path) {
  return await invoke('check_file_exists', { path });
}

/**
 * 检测设备当前推荐修补的启动分区
 * @returns {Promise<{
 *   slotSuffix: string,
 *   currentSlot: string,
 *   availablePartitions: string[],
 *   hasBoot: boolean,
 *   hasInitBoot: boolean,
 *   recommendedPartition: string,
 *   recommendedImageName: string,
 *   reason: string,
 *   warningLevel: string
 * }>}
 */
// =====================================================
// Apps 相关 API — 注入 serial
// =====================================================

/**
 * 获取包列表
 * @param {boolean} onlyThirdParty
 */
export async function fetchPackages(onlyThirdParty) {
  return await invoke('get_packages', { serial: getSerial(), onlyThirdParty });
}

/**
 * 获取已禁用包列表
 */
export async function fetchDisabledPackages() {
  return await invoke('get_disabled_packages', { serial: getSerial() });
}

/**
 * 获取单个包详细信息
 * @param {string} packageName
 */
export async function fetchPackageDetail(packageName) {
  return await invoke('get_package_detail', { serial: getSerial(), packageName });
}

/**
 * 获取前台应用
 */
export async function fetchForegroundPackage() {
  return await invoke('get_foreground_package', { serial: getSerial() });
}

/**
 * 应用管理操作
 * @param {string} packageName
 * @param {string} action
 */
export async function managePackage(packageName, action) {
  return await invoke('manage_package', { serial: getSerial(), packageName, action });
}

/**
 * 安装 APK
 * @param {string} apkPath
 * @param {boolean} reinstall
 * @param {boolean} downgrade
 */
export async function installPackage(apkPath, reinstall, downgrade) {
  return await invoke('install_package', { serial: getSerial(), apkPath, reinstall, downgrade });
}

/**
 * 批量获取应用标签
 * @param {boolean} onlyThirdParty
 */
export async function fetchAppLabels(onlyThirdParty) {
  return await invoke('get_app_labels', { serial: getSerial(), onlyThirdParty });
}

/**
 * 提取 APK
 * @param {string} packageName
 * @param {string} savePath
 */
export async function extractApk(packageName, savePath) {
  return await invoke('extract_apk', { serial: getSerial(), packageName, savePath });
}


// =====================================================
// 设备控制相关 API — Device Control
// =====================================================

/**
 * 设置屏幕分辨率
 * @param {number} width
 * @param {number} height
 */
export async function setResolution(width, height) {
  return await runAdbShell(['wm', 'size', `${width}x${height}`]);
}

/**
 * 重置屏幕分辨率为默认值
 */
export async function resetResolution() {
  return await runAdbShell(['wm', 'size', 'reset']);
}

/**
 * 获取当前屏幕分辨率
 * @returns {Promise<string>}
 */
export async function getResolution() {
  return await runAdbShell(['wm', 'size']);
}

/**
 * 设置屏幕显示密度 (DPI)
 * @param {number} dpi
 */
export async function setDensity(dpi) {
  return await runAdbShell(['wm', 'density', String(dpi)]);
}

/**
 * 重置显示密度为默认值
 */
export async function resetDensity() {
  return await runAdbShell(['wm', 'density', 'reset']);
}

/**
 * 获取当前显示密度
 * @returns {Promise<string>}
 */
export async function getDensity() {
  return await runAdbShell(['wm', 'density']);
}

/**
 * 设置模拟电量 (0-100)
 * @param {number} level
 */
export async function setBatteryLevel(level) {
  return await runAdbShell(['dumpsys', 'battery', 'set', 'level', String(level)]);
}

/**
 * 设置充电状态模拟
 * @param {boolean} charging - true: 充电中 (status=2), false: 未充电 (status=1)
 */
export async function setBatteryCharging(charging) {
  // status: 2=CHARGING, 1=DISCHARGING
  await runAdbShell(['dumpsys', 'battery', 'set', 'status', charging ? '2' : '1']);
  // plug: 2=AC, 0=UNPLUGGED
  await runAdbShell(['dumpsys', 'battery', 'set', 'plugged', charging ? '2' : '0']);
}

/**
 * 取消电池模拟，恢复真实数据
 */
export async function resetBattery() {
  return await runAdbShell(['dumpsys', 'battery', 'reset']);
}

/**
 * 读取当前电池真实信息（level、temperature 等原始文本）
 */
export async function getBatteryInfo() {
  return await runAdbShell(['dumpsys', 'battery']);
}

/**
 * 设置屏幕旋转 (需先禁用自动旋转)
 * @param {number} rotation - 0: 竖屏, 1: 左横, 2: 倒竖, 3: 右横
 */
export async function setRotation(rotation) {
  // 先禁用自动旋转
  await runAdbShell(['settings', 'put', 'system', 'accelerometer_rotation', '0']);
  // 再设置旋转方向
  return await runAdbShell(['settings', 'put', 'system', 'user_rotation', String(rotation)]);
}

/**
 * 设置是否启用自动旋转
 * @param {boolean} enabled
 */
export async function setAutoRotation(enabled) {
  return await runAdbShell(['settings', 'put', 'system', 'accelerometer_rotation', enabled ? '1' : '0']);
}

/**
 * 设置屏幕亮度 (0-255)
 * @param {number} value
 */
export async function setBrightness(value) {
  // 先关闭自动亮度
  await runAdbShell(['settings', 'put', 'system', 'screen_brightness_mode', '0']);
  return await runAdbShell(['settings', 'put', 'system', 'screen_brightness', String(value)]);
}

/**
 * 设置自动亮度开关
 * @param {boolean} enabled
 */
export async function setAutoBrightness(enabled) {
  return await runAdbShell(['settings', 'put', 'system', 'screen_brightness_mode', enabled ? '1' : '0']);
}

/**
 * 获取屏幕亮度 (0-255)
 */
export async function getBrightness() {
  const out = await runAdbShell(['settings', 'get', 'system', 'screen_brightness']);
  const val = parseInt(out.trim());
  return isNaN(val) ? 128 : val;
}

/**
 * 获取自动亮度开关状态
 */
export async function getAutoBrightness() {
  const out = await runAdbShell(['settings', 'get', 'system', 'screen_brightness_mode']);
  return out.trim() === '1';
}

/**
 * 设置指定音频流的音量
 * 优先使用 media volume 命令，失败时降级为 keyevent 模拟
 * @param {number} stream - 音频流编号 (3:通话, 4:音乐, 5:通知, 2:铃声)
 * @param {number} value  - 音量值 (0-15)
 */
export async function setVolumeStream(stream, value) {
  try {
    const result = await runAdbShell(['media', 'volume', '--stream', String(stream), '--set', String(value)]);
    // media volume 命令存在但没有报错认为成功
    if (!result.toLowerCase().includes('error')) return result;
    throw new Error('media volume failed');
  } catch {
    // 降级：通过 keyevent 模拟（仅支持媒体音量 stream=3）
    // 其他 stream 直接忽略降级
    if (stream === 3) {
      // 先设置为最小，再逐步增加（粗略方案，仅作降级）
      const steps = Math.round(value);
      for (let i = 0; i < 15; i++) {
        await runAdbShell(['input', 'keyevent', '25']); // KEYCODE_VOLUME_DOWN
      }
      for (let i = 0; i < steps; i++) {
        await runAdbShell(['input', 'keyevent', '24']); // KEYCODE_VOLUME_UP
      }
    }
  }
}

/**
 * 获取当前音量信息（通过 dumpsys audio）
 * @returns {Promise<string>}
 */
export async function getVolumeInfo() {
  return await runAdbShell(['dumpsys', 'audio']);
}

/**
 * 获取锁屏超时时间（毫秒 → 秒）
 */
export async function getLockScreenTimeout() {
  const out = await runAdbShell(['settings', 'get', 'secure', 'lock_screen_timeout']);
  const ms = parseInt(out.trim());
  return isNaN(ms) ? 0 : Math.round(ms / 1000);
}

/**
 * 设置锁屏超时时间
 * @param {number} seconds
 */
export async function setLockScreenTimeout(seconds) {
  return await runAdbShell(['settings', 'put', 'secure', 'lock_screen_timeout', String(seconds * 1000)]);
}

/**
 * 获取字体缩放比例
 */
export async function getFontScale() {
  const out = await runAdbShell(['settings', 'get', 'system', 'font_scale']);
  return parseFloat(out.trim()) || 1.0;
}

/**
 * 设置字体缩放比例
 * @param {number} scale
 */
export async function setFontScale(scale) {
  return await runAdbShell(['settings', 'put', 'system', 'font_scale', String(scale)]);
}

/**
 * 批量获取动画倍速
 * @returns {{ window: number, transition: number, animator: number }}
 */
export async function getAnimationScales() {
  const [w, t, a] = await Promise.all([
    runAdbShell(['settings', 'get', 'global', 'window_animation_scale']),
    runAdbShell(['settings', 'get', 'global', 'transition_animation_scale']),
    runAdbShell(['settings', 'get', 'global', 'animator_duration_scale']),
  ]);
  return {
    window:     parseFloat(w.trim()) || 1.0,
    transition: parseFloat(t.trim()) || 1.0,
    animator:   parseFloat(a.trim()) || 1.0,
  };
}

/**
 * 设置单项动画倍速
 * @param {'window_animation_scale'|'transition_animation_scale'|'animator_duration_scale'} type
 * @param {number} scale
 */
export async function setAnimationScale(type, scale) {
  return await runAdbShell(['settings', 'put', 'global', type, String(scale)]);
}

/**
 * 设置电池充电插接模式 (不依赖 charging=true/false，精细控制 plugged)
 * @param {'none'|'ac'|'usb'|'wireless'} mode
 */
export async function setBatteryPlugged(mode) {
  const pluggedMap = { none: '0', ac: '1', usb: '2', wireless: '4' };
  const plugged  = pluggedMap[mode] ?? '0';
  const status   = mode === 'none' ? '1' : '2'; // 1=DISCHARGING 2=CHARGING
  await runAdbShell(['dumpsys', 'battery', 'set', 'status',  status]);
  return await runAdbShell(['dumpsys', 'battery', 'set', 'plugged', plugged]);
}

/**
 * 读取状态栏图标黑名单（被隐藏的图标 key 数组）
 */
export async function getIconBlacklist() {
  const out = await runAdbShell(['settings', 'get', 'secure', 'icon_blacklist']);
  const raw = out.trim();
  return (raw === 'null' || raw === '') ? [] : raw.split(',').filter(Boolean);
}

/**
 * 写入状态栏图标黑名单
 * @param {string[]} iconKeys
 */
export async function setIconBlacklist(iconKeys) {
  if (!iconKeys || iconKeys.length === 0) {
    return await runAdbShell(['settings', 'delete', 'secure', 'icon_blacklist']);
  }
  return await runAdbShell(['settings', 'put', 'secure', 'icon_blacklist', iconKeys.join(',')]);
}

/**
 * 获取状态栏时钟是否始终显秒
 */
export async function getClockSeconds() {
  const out = await runAdbShell(['settings', 'get', 'secure', 'clock_seconds']);
  return out.trim() === '1';
}

/**
 * 设置状态栏时钟始终显秒
 * @param {boolean} enabled
 */
export async function setClockSeconds(enabled) {
  return await runAdbShell(['settings', 'put', 'secure', 'clock_seconds', enabled ? '1' : '0']);
}

/**
 * 模拟电池温度
 * @param {number} celsius - 摄氏度（内部乘以 10 传给 dumpsys）
 */
export async function setBatteryTemperature(celsius) {
  return await runAdbShell(['dumpsys', 'battery', 'set', 'temperature', String(Math.round(celsius * 10))]);
}

/**
 * 获取系统属性或设置列表
 * @param {'getprop'|'system'|'global'|'secure'} propType
 */
export async function fetchSysProps(propType) {
  return await invoke('get_sys_props', { serial: getSerial(), propType });
}

/**
 * 设置/新增系统属性或设置
 * @param {'getprop'|'system'|'global'|'secure'} propType
 * @param {string} key
 * @param {string} value
 */
export async function setSysProp(propType, key, value) {
  return await invoke('set_sys_prop', { serial: getSerial(), propType, key, value });
}

/**
 * 删除系统设置
 * @param {'getprop'|'system'|'global'|'secure'} propType
 * @param {string} key
 */
export async function deleteSysProp(propType, key) {
  return await invoke('delete_sys_prop', { serial: getSerial(), propType, key });
}
