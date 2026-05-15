<template>
  <div class="page-container no-scrollbar">
    <OverviewHardwareShowcase
      :loading="loadingInfo"
      :title="hardwareHeroTitle"
      :description="hardwareHeroDescription"
      :highlights="hardwareHighlights"
      :columns="hardwareColumns"
      @wireless-adb="openWirelessAdb"
    />

    <OverviewTopCards
      :cards="topCards"
      :loading-info="loadingInfo"
      :loading-app="loadingApp"
    />

    <div class="resource-row">
      <OverviewExtraInfo
        :cards="extraInfoCards"
        :loading="loadingInfo"
      />

      <OverviewResourcePanel
        :metrics="sideResourceMetrics"
        :battery-metric="batteryMetric"
        :battery-status-text="batteryStatusText"
        :battery-temp="resources.battery_temp"
        :loading="loadingRes"
      />
    </div>

    <OverviewCommandGrid
      :options="rebootOptions"
      :is-connected="isConnected"
      @reboot="handleReboot"
    />

    <WirelessAdbDialog
      ref="wirelessDialogRef"
      :show-float-log="true"
      :float-logs="scanLogs"
      @clear-float-logs="clearScanLogs"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import WirelessAdbDialog from '@/components/common/WirelessAdbDialog.vue';
import { fetchDeviceInfo, fetchAppStatus, fetchDeviceResources, deviceReboot } from '@/api/device';
import { useDeviceStore } from '@/utils/deviceStore';
import {
  OverviewTopCards,
  OverviewExtraInfo,
  OverviewResourcePanel,
  OverviewHardwareShowcase,
  OverviewCommandGrid,
} from './components';
import {
  OVERVIEW_FIELD_DEFINITIONS,
  OVERVIEW_SECTION_CONFIG,
  createDefaultOverviewDeviceInfo,
} from './config';

const loadingInfo = ref(false);
const loadingApp = ref(false);
const loadingRes = ref(false);
const deviceError = ref('');
const wirelessDialogRef = ref(null);
const {
  isConnected,
  scanLogs,
  clearScanLogs,
  startScanLogging,
  stopScanLogging,
} = useDeviceStore();

const resources = ref({
  storage_used_gb: 0,
  storage_total_gb: 0,
  storage_percent: 0,
  memory_used_gb: 0,
  memory_total_gb: 0,
  memory_percent: 0,
  battery_level: 0,
  battery_temp: 0,
});

const appStatus = ref({
  system_count: 0,
  user_count: 0,
  total_count: 0,
});

const deviceInfo = ref(createDefaultOverviewDeviceInfo());

const rebootOptions = [
  { label: '重启手机', icon: 'reboot', color: 'var(--color-info)', hoverBg: 'var(--info-soft)', target: 'system', desc: '返回系统模式' },
  { label: 'Fastboot 模式', icon: 'robot', color: 'var(--brand-lime)', hoverBg: 'var(--brand-lime-soft)', target: 'bootloader', desc: '进入 Bootloader 模式' },
  { label: 'Recovery 模式', icon: 'tool', color: 'var(--brand-rose)', hoverBg: 'rgba(var(--color-danger-rgb), 0.08)', target: 'recovery', desc: '进入恢复模式' },
  { label: '彻底关机', icon: 'shutdown', color: 'var(--color-danger)', hoverBg: 'var(--danger-soft)', target: 'poweroff', desc: '结束当前设备会话' },
];

const topCards = computed(() => {
  const infoCards = OVERVIEW_SECTION_CONFIG.topInfoCards.map((card) => ({
    ...card,
    label: card.label || OVERVIEW_FIELD_DEFINITIONS[card.key]?.label || card.key,
    value: deviceInfo.value[card.key] || '--',
  }));

  const appCards = OVERVIEW_SECTION_CONFIG.appCards.map((card) => ({
    ...card,
    value: appStatus.value[card.key] || '--',
  }));

  return [...infoCards, ...appCards];
});

const getFieldValue = (key) => deviceInfo.value[key] || OVERVIEW_FIELD_DEFINITIONS[key]?.fallback || '--';

const createFieldViewModel = (key) => ({
  key,
  label: OVERVIEW_FIELD_DEFINITIONS[key]?.label || key,
  value: getFieldValue(key),
});

const hardwareColumns = computed(() => (
  OVERVIEW_SECTION_CONFIG.hardwareColumns.map((group) => group.map((key) => createFieldViewModel(key)))
));

const hardwareHeroTitle = computed(() => '设备硬件画像');

const hardwareHeroDescription = computed(() => (
  '上方聚合快速识别信息，这里专注展示设备品牌画像与底层硬件特征，便于快速判断机型环境。'
));

const hardwareHighlights = computed(() => (
  OVERVIEW_SECTION_CONFIG.heroHighlights.map((item) => ({
    ...createFieldViewModel(item.key),
    label: item.label || OVERVIEW_FIELD_DEFINITIONS[item.key]?.label || item.key,
  }))
));

const extraInfoCards = computed(() => (
  OVERVIEW_SECTION_CONFIG.extraPanels.map((panel) => ({
    ...panel,
    value: getFieldValue(panel.key),
  }))
));

const batteryMetric = computed(() => ({
  label: '电池',
  percent: resources.value.battery_level || 0,
  color: resources.value.battery_level < 20
    ? 'var(--color-danger)'
    : resources.value.battery_level < 50
      ? 'var(--color-warning)'
      : 'var(--color-success)',
  accent: resources.value.battery_level < 20
    ? 'rgba(var(--color-danger-rgb), 0.18)'
    : resources.value.battery_level < 50
      ? 'rgba(var(--color-warning-rgb), 0.18)'
      : 'rgba(var(--color-success-rgb), 0.18)',
}));

const batteryStatusText = computed(() => {
  if (resources.value.battery_level < 20) return '低电量';
  if (resources.value.battery_level < 50) return '续航中';
  return '正常';
});

const sideResourceMetrics = computed(() => [
  {
    label: '存储',
    percent: resources.value.storage_percent || 0,
    meta: `${resources.value.storage_used_gb}/${resources.value.storage_total_gb}G`,
    icon: 'folder',
    color: 'var(--color-primary)',
    accent: 'rgba(var(--color-primary-rgb), 0.16)',
  },
  {
    label: '内存',
    percent: resources.value.memory_percent || 0,
    meta: `${resources.value.memory_used_gb}/${resources.value.memory_total_gb}G`,
    icon: 'system',
    color: 'var(--color-success)',
    accent: 'rgba(var(--color-success-rgb), 0.16)',
  },
]);

async function loadDeviceInfo() {
  loadingInfo.value = true;
  deviceError.value = '';
  try {
    const info = await fetchDeviceInfo();
    const formatUnlockState = (state) => {
      const normalized = String(state || '').toLowerCase();
      if (normalized === 'unlocked') return '已解锁 (Unlocked)';
      if (normalized === 'locked') return '已上锁 (Locked)';
      if (normalized === 'unknown') return '未知状态';
      return normalized || '--';
    };

    deviceInfo.value = {
      ...createDefaultOverviewDeviceInfo(),
      device_name: info.device_name || '--',
      device_codename: info.device_codename || '--',
      serial: info.serial || '--',
      state: info.state || '--',
      brand: info.brand || '--',
      android_version: info.android_version || '--',
      os_version: info.os_version || '--',
      cpu_codename: info.cpu_codename || '--',
      cpu_arch: info.cpu_arch || '--',
      hardware_platform: info.hardware_platform || '--',
      board_id: info.board_id || '--',
      resolution: info.resolution || '--',
      display_density: info.display_density || '--',
      unlock_state: formatUnlockState(info.unlock_state),
      ab_slot: info.ab_slot || '--',
      vndk_version: info.vndk_version || '--',
      uptime: info.uptime || '--',
      build_date: info.build_date || '--',
      build_version: info.build_version || '--',
      fingerprint: info.fingerprint || '--',
      kernel_version: info.kernel_version || '--',
    };
  } catch (e) {
    deviceError.value = String(e);
    deviceInfo.value = createDefaultOverviewDeviceInfo();
  } finally {
    loadingInfo.value = false;
  }
}

async function loadAppStatus() {
  loadingApp.value = true;
  try {
    appStatus.value = await fetchAppStatus();
  } catch (_) {
    appStatus.value = { system_count: 0, user_count: 0, total_count: 0 };
  } finally {
    loadingApp.value = false;
  }
}

async function handleReboot(opt) {
  try {
    await ElMessageBox.confirm(
      `确定要执行“${opt.label}”${opt.desc ? `，${opt.desc}` : ''}吗？`,
      '操作确认',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    await deviceReboot(opt.target);
    ElMessage.success(`${opt.label} 指令已发送`);
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`操作失败：${e}`);
    }
  }
}

async function loadResources() {
  loadingRes.value = true;
  try {
    resources.value = await fetchDeviceResources();
  } catch (_) {
    resources.value = {
      storage_used_gb: 0,
      storage_total_gb: 0,
      storage_percent: 0,
      memory_used_gb: 0,
      memory_total_gb: 0,
      memory_percent: 0,
      battery_level: 0,
      battery_temp: 0,
    };
  } finally {
    loadingRes.value = false;
  }
}

function openWirelessAdb() {
  if (wirelessDialogRef.value && typeof wirelessDialogRef.value.open === 'function') {
    wirelessDialogRef.value.open();
  }
}

async function loadAllData() {
  if (!isConnected.value) {
    deviceError.value = '没有已连接的设备';
    resetAllData();
    return;
  }
  deviceError.value = '';
  loadDeviceInfo();
  loadAppStatus();
  loadResources();
}

function resetAllData() {
  loadingInfo.value = false;
  loadingApp.value = false;
  loadingRes.value = false;

  deviceInfo.value = createDefaultOverviewDeviceInfo();

  resources.value = {
    storage_used_gb: 0,
    storage_total_gb: 0,
    storage_percent: 0,
    memory_used_gb: 0,
    memory_total_gb: 0,
    memory_percent: 0,
    battery_level: 0,
    battery_temp: 0,
  };

  appStatus.value = { system_count: 0, user_count: 0, total_count: 0 };
}

watch(isConnected, (connected) => {
  if (connected) {
    loadAllData();
  } else {
    deviceError.value = '设备已断开';
    resetAllData();
  }
}, { immediate: true });

defineExpose({
  refresh: loadAllData,
});
onMounted(() => {
  startScanLogging('overview');
});

onUnmounted(() => {
  stopScanLogging('overview');
});
</script>

<style lang="scss" scoped>

.resource-row {
  display: flex;
  gap: 16px;
  flex-shrink: 0;
  width: 100%;
  padding-bottom: 20px;
}

@media (max-width: 768px) {
  .resource-row {
    flex-direction: column;
  }
}
</style>
