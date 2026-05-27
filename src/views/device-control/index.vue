<template>
  <div class="device-control-page">
    <div class="top-row">
      <DeviceDisplayCard
        :display="display"
        :brightness="brightness"
        :status-bar="statusBar"
        :loading="loading.display"
        :is-connected="isConnected"
        @apply-display="handleApplyDisplay"
        @reset-display="handleResetDisplay"
        @set-brightness="handleSetBrightness"
        @set-auto-brightness="handleSetAutoBrightness"
        @toggle-clock-seconds="handleToggleClockSeconds"
      />

      <DeviceBatteryCard
        :battery="battery"
        :charge-modes="chargeModes"
        :is-connected="isConnected"
        @reset-battery="handleResetBattery"
        @set-battery-level="handleSetBatteryLevel"
        @set-battery-temperature="handleSetBatteryTemperature"
        @set-battery-mode="handleSetBatteryMode"
      />

      <DeviceLockscreenCard
        :lockscreen="lockscreen"
        :lock-presets="lockPresets"
        :quick-actions="quickActions"
        :is-connected="isConnected"
        @set-lockscreen="handleSetLockscreen"
        @keyevent="handleKeyevent"
      />
    </div>

    <DeviceStatusIconsCard
      :status-icons="statusIcons"
      :hidden-count="hiddenCount"
      :all-selected="allSelected"
      :is-connected="isConnected"
      @toggle-select-all="toggleSelectAll"
    />
  </div>
</template>

<script setup>
import { computed, reactive, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { debounce } from '@/utils/debounce';
import { useDeviceStore } from '@/utils/deviceStore';
import {
  setResolution,
  resetResolution,
  getResolution,
  setDensity,
  resetDensity,
  getDensity,
  setBatteryLevel,
  setBatteryTemperature,
  resetBattery,
  setBatteryPlugged,
  getBatteryInfo,
  getLockScreenTimeout,
  setLockScreenTimeout,
  setBrightness,
  setAutoBrightness,
  getBrightness,
  getAutoBrightness,
  getIconBlacklist,
  setIconBlacklist,
  getClockSeconds,
  setClockSeconds,
  sendKeyevent,
} from '@/api/device';
import { DeviceBatteryCard, DeviceDisplayCard, DeviceLockscreenCard, DeviceStatusIconsCard } from './components';
import { chargeModes, lockPresets, quickActions, statusIconItems } from './components/config';

const { isConnected, selectedSerial } = useDeviceStore();

const loading = reactive({ display: false });
const display = reactive({ width: '', height: '', density: '', curRes: '', curDensity: '' });
const brightness = reactive({ value: 128, auto: false });
const battery = reactive({ level: 100, temperature: 35, mode: 'ac' });
const lockscreen = reactive({ current: 0, inputSec: 60 });
const statusBar = reactive({ clockSeconds: false });
const statusIcons = reactive(statusIconItems.map((item) => ({ ...item, hidden: false })));

async function handleApplyDisplay() {
  loading.display = true;
  try {
    const width = parseInt(display.width, 10);
    const height = parseInt(display.height, 10);
    if (width && height) {
      await setResolution(width, height);
    }

    const density = parseInt(display.density, 10);
    if (density) {
      await setDensity(density);
    }

    ElMessage.success('显示设置已应用');
    await loadDisplayInfo();
  } catch (error) {
    ElMessage.error(`应用失败: ${error}`);
  } finally {
    loading.display = false;
  }
}

async function handleResetDisplay() {
  try {
    await Promise.all([resetResolution(), resetDensity()]);
    ElMessage.success('显示已恢复默认');
    await loadDisplayInfo();
  } catch (error) {
    ElMessage.error(`恢复失败: ${error}`);
  }
}

async function handleSetBrightness(value) {
  try {
    await setBrightness(value);
  } catch (error) {
    ElMessage.error(`调节亮度失败: ${error}`);
  }
}

async function handleSetAutoBrightness(value) {
  try {
    await setAutoBrightness(value);
  } catch (error) {
    ElMessage.error(`设置自动亮度失败: ${error}`);
  }
}

async function handleSetBatteryLevel() {
  try {
    await setBatteryLevel(battery.level);
  } catch (error) {
    ElMessage.error(`设置失败: ${error}`);
  }
}

async function handleSetBatteryTemperature() {
  try {
    await setBatteryTemperature(battery.temperature);
  } catch (error) {
    ElMessage.error(`设置温度失败: ${error}`);
  }
}

async function handleSetBatteryMode(mode) {
  battery.mode = mode;
  try {
    await setBatteryPlugged(mode);
  } catch (error) {
    ElMessage.error(`设置充电模式失败: ${error}`);
  }
}

async function handleResetBattery() {
  try {
    await resetBattery();
    ElMessage.success('电池模拟已恢复');
    await loadBatteryInfo();
  } catch (error) {
    ElMessage.error(`恢复失败: ${error}`);
  }
}

async function handleSetLockscreen() {
  try {
    const seconds = lockscreen.inputSec === 0 ? 2147483 : lockscreen.inputSec;
    await setLockScreenTimeout(seconds);
    lockscreen.current = lockscreen.inputSec === 0 ? -1 : lockscreen.inputSec;
    ElMessage.success('锁屏时间已更新');
  } catch (error) {
    ElMessage.error(`设置失败: ${error}`);
  }
}

const allSelected = computed(() => statusIcons.every((item) => item.hidden));
const hiddenCount = computed(() => statusIcons.filter((item) => item.hidden).length);

const applyIconBlacklist = debounce(async () => {
  if (!isConnected.value) {
    return;
  }

  try {
    const keys = statusIcons.filter((item) => item.hidden).map((item) => item.key);
    await setIconBlacklist(keys);
  } catch (error) {
    ElMessage.error(`应用失败: ${error}`);
  }
}, 500);

watch(
  () => statusIcons.map((item) => item.hidden),
  applyIconBlacklist,
  { deep: true }
);

function toggleSelectAll() {
  const next = !allSelected.value;
  statusIcons.forEach((item) => {
    item.hidden = next;
  });
}

async function handleToggleClockSeconds(value) {
  try {
    await setClockSeconds(value);
  } catch (error) {
    ElMessage.error(`设置失败: ${error}`);
  }
}

async function handleKeyevent(code) {
  try {
    await sendKeyevent(code);
  } catch (error) {
    ElMessage.error(`按键执行失败: ${error}`);
  }
}

async function loadDisplayInfo() {
  if (!isConnected.value) {
    return;
  }

  try {
    const [resolutionOutput, densityOutput] = await Promise.all([getResolution(), getDensity()]);
    const sizeLine = resolutionOutput.split('\n').find((line) => line.includes('Override size') || line.includes('Physical size'));
    if (sizeLine) {
      const raw = sizeLine.split(':')[1]?.trim() || '';
      display.curRes = raw;
      const parts = raw.split('x');
      if (parts.length === 2) {
        display.width = parts[0];
        display.height = parts[1];
      }
    }

    const densityLine = densityOutput.split('\n').find((line) => line.includes('Override density') || line.includes('Physical density'));
    if (densityLine) {
      const value = densityLine.split(':')[1]?.trim() || '';
      display.curDensity = value;
      display.density = value;
    }
  } catch {
    // 读取失败时保持静默，避免打断页面操作。
  }
}

async function loadBatteryInfo() {
  if (!isConnected.value) {
    return;
  }

  try {
    const output = await getBatteryInfo();
    const levelLine = output.split('\n').find((line) => line.includes('level:'));
    if (levelLine) {
      const value = parseInt(levelLine.split(':')[1]?.trim(), 10);
      if (!Number.isNaN(value)) {
        battery.level = value;
      }
    }

    const temperatureLine = output.split('\n').find((line) => line.includes('temperature:'));
    if (temperatureLine) {
      const value = parseInt(temperatureLine.split(':')[1]?.trim(), 10);
      if (!Number.isNaN(value)) {
        battery.temperature = Math.round(value / 10);
      }
    }
  } catch {
    // 忽略读取失败，保留当前 UI 状态。
  }
}

async function loadLockscreenInfo() {
  if (!isConnected.value) {
    return;
  }

  try {
    const seconds = await getLockScreenTimeout();
    lockscreen.current = seconds;
    if (seconds > 0) {
      lockscreen.inputSec = seconds;
    }
  } catch {
    // 忽略读取失败。
  }
}

async function loadIconBlacklist() {
  if (!isConnected.value) {
    return;
  }

  try {
    const keys = await getIconBlacklist();
    statusIcons.forEach((icon) => {
      icon.hidden = keys.includes(icon.key);
    });
  } catch {
    // 忽略读取失败。
  }
}

async function loadClockSeconds() {
  if (!isConnected.value) {
    return;
  }

  try {
    statusBar.clockSeconds = await getClockSeconds();
  } catch {
    // 忽略读取失败。
  }
}

async function loadBrightnessInfo() {
  if (!isConnected.value) {
    return;
  }

  try {
    const [value, auto] = await Promise.all([getBrightness(), getAutoBrightness()]);
    brightness.value = value;
    brightness.auto = auto;
  } catch {
    // 忽略读取失败。
  }
}

async function refresh() {
  await Promise.all([
    loadDisplayInfo(),
    loadLockscreenInfo(),
    loadIconBlacklist(),
    loadClockSeconds(),
    loadBatteryInfo(),
    loadBrightnessInfo(),
  ]);
}

watch(selectedSerial, () => {
  if (isConnected.value) {
    refresh();
  }
}, { immediate: true });

defineExpose({ refresh });
</script>

<style lang="scss" scoped>
.device-control-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  // min-height: 100%;
  // overflow: visible;
  padding-right: 4px;
}

.top-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  flex-shrink: 0;
}

.top-row > :nth-child(1) {
  --page-enter-delay: 0ms;
}

.top-row > :nth-child(2) {
  --page-enter-delay: 40ms;
}

.top-row > :nth-child(3) {
  --page-enter-delay: 80ms;
}

.device-control-page > :last-child {
  --page-enter-delay: 120ms;
}

.device-control-page > :first-child {
  margin-top: 4px;
}

</style>
