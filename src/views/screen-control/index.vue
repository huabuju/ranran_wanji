<template>
  <div class="screen-control-container">
    <div class="control-layout">
      <div class="left-column">
        <ScreenLaunchCard :is-launching="isLaunching" :is-streaming="isStreaming" :is-connected="isConnected" @launch="handleLaunch" @stop="handleStop" />
        <ScreenBasicSettingsCard
          :form="form"
          :is-streaming="isStreaming"
          :res-marks="resMarks"
          :fps-marks="fpsMarks"
          :bitrate-marks="bitrateMarks"
        />
        <ScreenShortcutCard :actions="shortcutActions" :is-streaming="isStreaming" @keyevent="handleKeyevent" />
      </div>

      <div class="right-column">
        <ScreenAdvancedSettingsCard :form="form" :items="advancedSwitches" :is-streaming="isStreaming" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ElMessage } from 'element-plus';
import { sendKeyevent } from '@/api/device';
import { useDeviceStore } from '@/utils/deviceStore';
import { useScrcpyStore } from '@/utils/scrcpyStore';
import { ScreenAdvancedSettingsCard, ScreenBasicSettingsCard, ScreenLaunchCard, ScreenShortcutCard } from './components';
import { advancedSwitches, bitrateMarks, fpsMarks, resMarks, shortcutActions } from './components/config';

const { isConnected, checkDeviceConnection } = useDeviceStore();
const { isLaunching, isStreaming, form, handleLaunch, handleStop } = useScrcpyStore();

async function handleKeyevent(code) {
  try {
    await sendKeyevent(code);
  } catch (error) {
    ElMessage.error(error.toString() || '指令执行失败');
  }
}

async function refresh() {
  await checkDeviceConnection();
}

defineExpose({ refresh });
</script>

<style lang="scss" scoped>
.screen-control-container {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 4px 0;
  overflow: visible;
  background: transparent;
}

.control-layout {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 20px;
}

.left-column,
.right-column {
  display: flex;
  flex-direction: column;
  gap: 20px;
  height: 100%;
}

@media (max-width: 900px) {
  .control-layout {
    grid-template-columns: 1fr;
  }
}
</style>
