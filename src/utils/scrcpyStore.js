import { ref, computed, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { startScrcpy, stopScrcpy } from '@/api/scrcpy';
import { useDeviceStore } from '@/utils/deviceStore';

const isLaunching = ref(false);
const isStreaming = ref(false);
const manualStopRequested = ref(false);
const autoMirrorBlockedSerial = ref('');

let autoMirrorTimer = null;
let autoMirrorInitialized = false;

const AUTO_MIRROR_DELAY_MS = 1500;

const form = ref({
  maxSizeIndex: 0,
  maxFpsIndex: 0,
  bitRateIndex: 2,
  screenOff: false,
  stayAwake: false,
  readOnly: false,
  showTouches: false,
  alwaysOnTop: false,
  fullscreen: false,
  autoMirror: false,
});

const saved = localStorage.getItem('scrcpySettings');
if (saved) {
  try {
    Object.assign(form.value, JSON.parse(saved));
  } catch (error) {
    console.error('Failed to parse scrcpySettings:', error);
  }
}

watch(form, (value) => {
  localStorage.setItem('scrcpySettings', JSON.stringify(value));
}, { deep: true });

const RES_VALUES = ['0', '800', '1280', '1920'];
const FPS_VALUES = ['0', '30', '60', '120'];
const BITRATE_VALUES = ['2M', '4M', '8M', '16M'];

const scrcpyArgs = computed(() => {
  const args = [];
  const maxSize = RES_VALUES[form.value.maxSizeIndex];
  const maxFps = FPS_VALUES[form.value.maxFpsIndex];
  const bitRate = BITRATE_VALUES[form.value.bitRateIndex];

  if (maxSize !== '0') args.push('-m', maxSize);
  if (maxFps !== '0') args.push('--max-fps', maxFps);
  if (bitRate) args.push('-b', bitRate);

  if (form.value.screenOff) args.push('--turn-screen-off');
  if (form.value.stayAwake) args.push('--stay-awake');
  if (form.value.readOnly) args.push('--no-control');
  if (form.value.showTouches) args.push('--show-touches');
  if (form.value.alwaysOnTop) args.push('--always-on-top');
  if (form.value.fullscreen) args.push('--fullscreen');

  return args;
});

function clearAutoMirrorTimer() {
  if (!autoMirrorTimer) return;
  clearTimeout(autoMirrorTimer);
  autoMirrorTimer = null;
}

function resetAutoMirrorBlock() {
  autoMirrorBlockedSerial.value = '';
}

async function handleLaunch() {
  if (isStreaming.value || isLaunching.value) return;

  clearAutoMirrorTimer();
  isLaunching.value = true;
  manualStopRequested.value = false;
  resetAutoMirrorBlock();

  try {
    await startScrcpy(scrcpyArgs.value);
    isStreaming.value = true;

    if (typeof ElMessage !== 'undefined') {
      ElMessage.success('投屏已启动');
    }
  } catch (error) {
    if (typeof ElMessage !== 'undefined') {
      ElMessage.error(error?.toString?.() || '启动失败');
    }
  } finally {
    isLaunching.value = false;
  }
}

async function handleStop() {
  manualStopRequested.value = true;
  resetAutoMirrorBlock();
  clearAutoMirrorTimer();

  try {
    await stopScrcpy();
    isStreaming.value = false;

    if (typeof ElMessage !== 'undefined') {
      ElMessage.success('已发送停止指令');
    }
  } catch (error) {
    if (typeof ElMessage !== 'undefined') {
      ElMessage.error(error?.toString?.() || '停止失败');
    }
  }
}

export function initGlobalAutoMirror() {
  if (autoMirrorInitialized) return;
  autoMirrorInitialized = true;

  const { selectedDevice } = useDeviceStore();

  const readyAutoMirrorSerial = computed(() => {
    const device = selectedDevice.value;
    if (!device || device.state !== 'device') return '';
    return device.serial || '';
  });

  const shouldAutoMirrorLaunch = computed(() => (
    form.value.autoMirror
    && !manualStopRequested.value
    && !!readyAutoMirrorSerial.value
    && !isStreaming.value
    && !isLaunching.value
    && autoMirrorBlockedSerial.value !== readyAutoMirrorSerial.value
  ));

  function scheduleAutoMirrorLaunch() {
    clearAutoMirrorTimer();
    autoMirrorTimer = setTimeout(async () => {
      autoMirrorTimer = null;

      const readySerial = readyAutoMirrorSerial.value;
      if (!readySerial || !shouldAutoMirrorLaunch.value) {
        return;
      }

      await handleLaunch();

      if (!isStreaming.value) {
        autoMirrorBlockedSerial.value = readySerial;
      }
    }, AUTO_MIRROR_DELAY_MS);
  }

  listen('scrcpy-exited', () => {
    isStreaming.value = false;
    isLaunching.value = false;

    if (!manualStopRequested.value) {
      resetAutoMirrorBlock();
    }
  });

  watch(readyAutoMirrorSerial, (newVal, oldVal) => {
    if (newVal !== oldVal) {
      resetAutoMirrorBlock();
    }
  });

  watch(shouldAutoMirrorLaunch, (shouldLaunch) => {
    if (!shouldLaunch) {
      clearAutoMirrorTimer();
      return;
    }

    scheduleAutoMirrorLaunch();
  }, { immediate: true });

  watch(() => form.value.autoMirror, (enabled, oldVal) => {
    if (enabled && !oldVal) {
      manualStopRequested.value = false;
      resetAutoMirrorBlock();
    }
  });
}

export function useScrcpyStore() {
  return {
    isLaunching,
    isStreaming,
    form,
    scrcpyArgs,
    handleLaunch,
    handleStop,
  };
}
