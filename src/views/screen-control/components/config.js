export const resMarks = { 0: 'Auto', 1: '480p', 2: '720p', 3: '1080p' };
export const fpsMarks = { 0: '无限制', 1: '30 FPS', 2: '60 FPS', 3: '120 FPS' };
export const bitrateMarks = { 0: '2 Mbps', 1: '4 Mbps', 2: '8 Mbps', 3: '16 Mbps' };

export const shortcutActions = [
  { code: 26, title: '电源', icon: 'shutdown', color: 'var(--color-danger)' },
  { code: 4, title: '返回', icon: 'back', color: 'var(--color-primary)' },
  { code: 3, title: '主页', icon: 'home', color: 'var(--color-success)' },
  { code: 187, title: '多任务', icon: 'recents', color: 'var(--color-info)' },
];

export const advancedSwitches = [
  {
    key: 'autoMirror',
    title: '连接后自动投屏',
    subtitle: 'Auto-Mirror on Connect',
    description: '设备接入并被识别后自动启动投屏',
    icon: 'bolt',
    color: 'var(--color-warning)',
    colorRgb: 'var(--color-warning-rgb)',
  },
  {
    key: 'screenOff',
    title: '熄屏投屏',
    subtitle: 'Turn Screen Off',
    description: '镜像时关闭设备物理屏幕，节省电量并保护隐私',
    icon: 'shutdown',
    color: 'var(--brand-violet)',
    colorRgb: '168, 85, 247',
  },
  {
    key: 'stayAwake',
    title: '保持设备唤醒',
    subtitle: 'Stay Awake',
    description: '连接期间防止设备进入休眠状态',
    icon: 'monitor',
    color: 'var(--color-info)',
    colorRgb: 'var(--color-info-rgb)',
  },
  {
    key: 'readOnly',
    title: '只读模式',
    subtitle: 'Read-only',
    description: '仅查看屏幕，禁用所有鼠标和键盘控制',
    icon: 'tool',
    color: 'var(--color-danger)',
    colorRgb: 'var(--color-danger-rgb)',
  },
  {
    key: 'showTouches',
    title: '显示屏幕触摸轨迹',
    subtitle: 'Show Touches',
    description: '在设备屏幕上显示物理和虚拟触摸操作',
    icon: 'recents',
    color: 'var(--brand-pink)',
    colorRgb: '236, 72, 153',
  },
  {
    key: 'alwaysOnTop',
    title: '窗口置顶',
    subtitle: 'Always On Top',
    description: '投屏窗口始终置于所有窗口顶层',
    icon: 'home',
    color: 'var(--color-success)',
    colorRgb: 'var(--color-success-rgb)',
  },
  {
    key: 'fullscreen',
    title: '全屏启动',
    subtitle: 'Fullscreen',
    description: '启动时直接进入全屏模式',
    icon: 'monitor',
    color: 'var(--color-primary)',
    colorRgb: 'var(--color-primary-rgb)',
  },
];