<template>
  <div class="page-container no-scrollbar">
    <OverviewSummaryPanel
      class="overview-summary-panel"
      :loading="loadingInfo || loadingApp || loadingRes"
      :resource-metrics="summaryResourceMetrics"
      :base-items="summaryBaseItems"
      :command-options="rebootOptions"
      :is-connected="isConnected"
      @reboot="handleReboot"
      @wireless-adb="openWirelessAdb"
    />

    <OverviewInspectionReport
      :loading="loadingInfo || loadingApp"
      :items="reportItems"
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
  OverviewSummaryPanel,
  OverviewInspectionReport,
} from './components';
import {
  OVERVIEW_REPORT_FIELD_DEFINITIONS,
  OVERVIEW_REPORT_FIELDS,
  OVERVIEW_SUMMARY_FIELDS,
  createDefaultOverviewDeviceInfo,
} from './config';

const loadingInfo = ref(false);
const loadingApp = ref(false);
const loadingRes = ref(false);
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

const summaryResourceMetrics = computed(() => [
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
  {
    label: '电池',
    percent: batteryMetric.value.percent,
    meta: `${batteryStatusText.value} · ${resources.value.battery_temp}°C`,
    icon: 'battery_charging',
    color: batteryMetric.value.color,
    accent: batteryMetric.value.accent,
  },
]);

const getAppStatusValue = (key) => {
  if (!isConnected.value) return '--';
  return appStatus.value[key] ?? '--';
};

const getReportValue = (key) => {
  if (key === 'system_count' || key === 'user_count' || key === 'total_count') {
    return getAppStatusValue(key);
  }

  return deviceInfo.value[key] || OVERVIEW_REPORT_FIELD_DEFINITIONS[key]?.fallback || '--';
};

const createReportItem = (key) => {
  const value = getReportValue(key);

  return {
    key,
    label: OVERVIEW_REPORT_FIELD_DEFINITIONS[key]?.label || key,
    currentValue: value,
  };
};

const summaryBaseItems = computed(() => (
  OVERVIEW_SUMMARY_FIELDS.map((key) => ({
    key,
    label: OVERVIEW_REPORT_FIELD_DEFINITIONS[key]?.label || key,
    value: getReportValue(key),
  }))
));

const reportItems = computed(() => (
  OVERVIEW_REPORT_FIELDS.map((key) => createReportItem(key))
));

async function loadDeviceInfo() {
  loadingInfo.value = true;
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
      manufacturer: info.manufacturer || '--',
      product_model: info.product_model || '--',
      product_name: info.product_name || '--',
      security_patch: info.security_patch || '--',
      vendor_security_patch: info.vendor_security_patch || '--',
      build_incremental: info.build_incremental || '--',
      build_type: info.build_type || '--',
      build_tags: info.build_tags || '--',
      baseband_version: info.baseband_version || '--',
      soc_manufacturer: info.soc_manufacturer || '--',
      soc_model: info.soc_model || '--',
      cpu_abilist: info.cpu_abilist || '--',
    };
  } catch (_) {
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
    resetAllData();
    return;
  }
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
.page-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: 100%;
  flex-shrink: 0;
  // padding-bottom: 48px;
}

.no-scrollbar {
  overflow: visible;
}

.overview-summary-panel {
  width: 100%;
  flex-shrink: 0;
}

@media (prefers-reduced-motion: reduce) {
  .page-container :deep(.dashboard-card) {
    animation: none !important;
  }
}
</style>
