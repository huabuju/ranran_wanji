export const OVERVIEW_FIELD_DEFINITIONS = {
  device_name: { label: '设备名称', fallback: '--' },
  device_codename: { label: '设备代号', fallback: '--' },
  serial: { label: '序列号', fallback: '--' },
  state: { label: '设备状态', fallback: '--' },
  brand: { label: '品牌', fallback: '--' },
  android_version: { label: '安卓版本', fallback: '--' },
  os_version: { label: '系统版本', fallback: '--' },
  cpu_codename: { label: 'CPU 代号', fallback: '--' },
  cpu_arch: { label: 'CPU 架构', fallback: '--' },
  hardware_platform: { label: '硬件平台', fallback: '--' },
  board_id: { label: '主板 ID', fallback: '--' },
  resolution: { label: '分辨率', fallback: '--' },
  display_density: { label: '显示密度', fallback: '--' },
  unlock_state: { label: '解锁状态', fallback: '--' },
  ab_slot: { label: 'AB 分区', fallback: '--' },
  vndk_version: { label: 'VNDK 版本', fallback: '--' },
  uptime: { label: '开机时长', fallback: '--' },
  build_date: { label: '构建日期', fallback: '--' },
  build_version: { label: '编译版本', fallback: '--' },
  fingerprint: { label: '设备指纹', fallback: '--' },
  kernel_version: { label: '内核版本', fallback: '--' },
};

export const OVERVIEW_SECTION_CONFIG = {
  topInfoCards: [
    { key: 'device_name', icon: 'device', color: 'var(--brand-blue)', bgColor: 'var(--brand-blue-soft)' },
    { key: 'device_codename', icon: 'cpu', color: 'var(--color-success)', bgColor: 'var(--success-soft)' },
    { key: 'serial', icon: 'serial', color: 'var(--brand-primary-strong)', bgColor: 'var(--color-primary-light)' },
    { key: 'state', icon: 'power', color: 'var(--color-warning)', bgColor: 'var(--warning-soft)' },
    { key: 'android_version', icon: 'android', color: 'var(--color-success)', bgColor: 'var(--success-soft)' },
    { key: 'ab_slot', icon: 'system', color: 'var(--status-recovery)', bgColor: 'var(--brand-purple-soft)', label: '当前槽位' },
  ],
  appCards: [
    { key: 'system_count', label: '系统应用', icon: 'package', color: 'var(--color-warning)', bgColor: 'var(--warning-soft)' },
    { key: 'user_count', label: '用户应用', icon: 'user', color: 'var(--color-info)', bgColor: 'var(--info-soft)' },
  ],
  heroHighlights: [
    { key: 'brand' },
    { key: 'os_version', label: '系统' },
    { key: 'kernel_version' },
  ],
  extraPanels: [
    {
      key: 'fingerprint',
      icon: 'serial',
      title: '设备指纹 (Fingerprint)',
      subtitle: '用于快速识别当前设备构建来源与系统指纹',
    },
    {
      key: 'uptime',
      icon: 'time',
      title: '系统运行时长 (Uptime)',
      subtitle: '记录设备连续开机运行时间',
    },
  ],
  hardwareColumns: [
    ['unlock_state', 'cpu_codename', 'cpu_arch', 'hardware_platform', 'board_id'],
    ['resolution', 'display_density', 'vndk_version', 'build_date', 'build_version'],
  ],
};

export function createDefaultOverviewDeviceInfo() {
  return Object.fromEntries(
    Object.entries(OVERVIEW_FIELD_DEFINITIONS).map(([key, definition]) => [key, definition.fallback])
  );
}
