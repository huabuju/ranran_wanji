<template>
  <div class="screen-control-container">
    <div class="control-layout">
      <div class="left-column">
        <ScreenLaunchCard
          :is-launching="isLaunching"
          :is-streaming="isStreaming"
          :is-connected="isConnected"
          @launch="handleLaunch"
          @stop="handleStop"
        />
        <ScreenBasicSettingsCard
          :form="form"
          :is-streaming="isStreaming"
          :res-marks="resMarks"
          :fps-marks="fpsMarks"
          :bitrate-marks="bitrateMarks"
        />
        <ScreenShortcutCard
          :actions="shortcutActions"
          :is-streaming="isStreaming"
          @keyevent="handleKeyevent"
        />
      </div>

      <div class="right-column">
        <ScreenAdvancedSettingsCard
          :form="form"
          :items="advancedSwitches"
          :is-streaming="isStreaming"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ElMessage } from "element-plus";
import { sendKeyevent } from "@/api/device";
import { useDeviceStore } from "@/utils/deviceStore";
import { useScrcpyStore } from "@/utils/scrcpyStore";
import {
  ScreenAdvancedSettingsCard,
  ScreenBasicSettingsCard,
  ScreenLaunchCard,
  ScreenShortcutCard,
} from "./components";
import {
  advancedSwitches,
  bitrateMarks,
  fpsMarks,
  resMarks,
  shortcutActions,
} from "./components/config";

const { isConnected, checkDeviceConnection } = useDeviceStore();
const { isLaunching, isStreaming, form, handleLaunch, handleStop } =
  useScrcpyStore();

async function handleKeyevent(code) {
  try {
    await sendKeyevent(code);
  } catch (error) {
    ElMessage.error(error.toString() || "指令执行失败");
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
  gap: 18px;
  padding: 4px 0;
  overflow: visible;
  background: transparent;
  animation: page-fade-in 0.5s ease-out;
}

.control-layout {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 18px;
}

.left-column,
.right-column {
  display: flex;
  flex-direction: column;
  gap: 18px;
  height: 100%;
}

.left-column > *,
.right-column > * {
  animation: card-rise 0.55s cubic-bezier(0.22, 1, 0.36, 1) both;
}

.left-column > *:nth-child(1) {
  animation-delay: 0.08s;
}
.left-column > *:nth-child(2) {
  animation-delay: 0.18s;
}
.right-column > *:nth-child(1) {
  animation-delay: 0.14s;
}

@keyframes page-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes card-rise {
  from {
    opacity: 0;
    transform: translateY(14px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 900px) {
  .control-layout {
    grid-template-columns: 1fr;
  }
}

@media (prefers-reduced-motion: reduce) {
  .screen-control-container,
  .left-column > *,
  .right-column > * {
    animation: none !important;
  }
}
</style>
