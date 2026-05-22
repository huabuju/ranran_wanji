<template>
  <div class="boot-patch-page">
    <BootPatchHeroPanel
      :mode-state="modeState"
      :execution-state="executionState"
      :patch-flags="patchFlags"
      :source-state="sourceState"
      :tool-state="toolState"
      :kernel-su-state="kernelSuState"
      :root-state="rootState"
      @update:keep-verity="form.keepVerity = $event"
      @update:keep-force-encrypt="form.keepForceEncrypt = $event"
      @update:patch-vbmeta-flag="form.patchVbmetaFlag = $event"
      @update:recovery-mode="form.recoveryMode = $event"
      @update:patch-mode="form.patchMode = $event"
      @select-boot="selectBootFile"
      @update:boot-path="form.bootPath = $event"
      @update:magisk-apk-path="form.magiskApkPath = $event"
      @update:apatch-apk-path="form.apatchApkPath = $event"
      @update:apatch-super-key="form.apatchSuperKey = $event"
      @update:kernel-su-path="form.kernelSuPath = $event"
      @update:kernel-su-kmi="form.kernelSuKmi = $event"
      @update:kernel-su-allow-shell="form.kernelSuAllowShell = $event"
      @update:kernel-su-enable-adbd="form.kernelSuEnableAdbd = $event"
      @select-output="selectOutputDir"
      @start="startPatch"
      @one-key-root="startOneKeyRoot"
      @reset="resetForm"
    />

    <FloatingLog
      ref="floatingLogRef"
      :logs="logs"
      :status="status"
      :steps="patchSteps"
      :output-path="outputPath"
      :package-zip-path="packageZipPath"
      :image-meta-preview="imageMetaPreview"
      :remote-work-dir="remoteWorkDir"
      @clear="logs = []"
      @open-package-dir="openPackageZipDir(packageZipPath)"
    />
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { ElMessage } from 'element-plus';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { useDeviceStore } from '@/utils/deviceStore';
import { getSystemDownloadDir } from '@/utils/systemPaths';
import { BootPatchHeroPanel } from './components';
import {
  PATCH_MODE_OPTIONS,
  isApatchMode,
  isKernelSuMode,
  normalizePatchMode,
} from './composables/bootPatchMode';
import { useBootPatchToolOptions } from './composables/useBootPatchToolOptions';
import { useBootPatchWorkflow } from './composables/useBootPatchWorkflow';
import { useKernelSuRuntime } from './composables/useKernelSuRuntime';

const { isConnected, selectedSerial } = useDeviceStore();
const form = reactive({
  patchMode: 'kernelsu',
  bootPath: '',
  magiskApkPath: '',
  apatchApkPath: '',
  apatchSuperKey: '',
  kernelSuPath: '',
  kernelSuKmi: '',
  outputDir: '',
  keepVerity: false,
  keepForceEncrypt: false,
  patchVbmetaFlag: false,
  recoveryMode: false,
  kernelSuAllowShell: false,
  kernelSuEnableAdbd: false,
  cleanupRemote: true,
});

const patching = ref(false);
const rooting = ref(false);
const defaultOutputDir = ref('');
const floatingLogRef = ref(null);
const skipNextKernelSuRuntimeAutoRefresh = ref(false);
let unlistenLog = null;

const {
  currentApatchToolOptions,
  currentApatchToolDir,
  currentMagiskToolOptions,
  currentMagiskToolDir,
  currentKernelToolOptions,
  currentKernelToolDir,
  getApatchToolOptionsByMode,
  getMagiskToolOptionsByMode,
  getKernelToolOptionsByMode,
  getDefaultMagiskToolPath,
  getDefaultApatchToolPath,
  getDefaultKernelSuToolPath,
  getEffectivePatchToolPath,
  getInitialPatchMode,
  loadPatchToolOptions,
} = useBootPatchToolOptions(form);

let workflowApi;
const {
  kernelSuRuntimeLoading,
  kernelSuKmiOptions,
  kernelSuDetectedKmi,
  kernelSuSuggestedKmi,
  refreshKernelSuRuntime,
} = useKernelSuRuntime({
  form,
  patching,
  rooting,
  addLocalLog: (...args) => workflowApi?.addLocalLog(...args),
});

workflowApi = useBootPatchWorkflow({
  form,
  isConnected,
  patching,
  rooting,
  defaultOutputDir,
  floatingLogRef,
  kernelSuKmiOptions,
  kernelSuSuggestedKmi,
  getApatchToolOptionsByMode,
  getMagiskToolOptionsByMode,
  getKernelToolOptionsByMode,
  getDefaultMagiskToolPath,
  getDefaultApatchToolPath,
  getDefaultKernelSuToolPath,
  getEffectivePatchToolPath,
  getInitialPatchMode,
  onRootRebooted: () => {
    skipNextKernelSuRuntimeAutoRefresh.value = true;
  },
});

const {
  logs,
  status,
  outputPath,
  packageZipPath,
  imageMetaPreview,
  remoteWorkDir,
  canPatch,
  canRoot,
  rootTargetPartition,
  rootPatchToolLabel,
  patchSteps,
  openPackageZipDir,
  selectBootFile,
  selectOutputDir,
  startPatch,
  startOneKeyRoot,
  resetForm,
  ensureApatchSuperKeyFilled,
  handleBootPatchLog,
} = workflowApi;

const modeState = computed(() => ({
  patchMode: form.patchMode,
  patchModeOptions: PATCH_MODE_OPTIONS,
}));
const executionState = computed(() => ({
  canPatch: canPatch.value,
  canRoot: canRoot.value,
  patching: patching.value,
  rooting: rooting.value,
  kernelSuRuntimeLoading: kernelSuRuntimeLoading.value,
  isConnected: isConnected.value,
}));
const patchFlags = computed(() => ({
  keepVerity: form.keepVerity,
  keepForceEncrypt: form.keepForceEncrypt,
  patchVbmetaFlag: form.patchVbmetaFlag,
  recoveryMode: form.recoveryMode,
  kernelSuAllowShell: form.kernelSuAllowShell,
  kernelSuEnableAdbd: form.kernelSuEnableAdbd,
}));
const sourceState = computed(() => ({
  bootPath: form.bootPath,
  outputDir: form.outputDir,
}));
const toolState = computed(() => ({
  magiskApkPath: form.magiskApkPath,
  magiskApkOptions: currentMagiskToolOptions.value,
  magiskApkDir: currentMagiskToolDir.value,
  apatchApkPath: form.apatchApkPath,
  apatchApkOptions: currentApatchToolOptions.value,
  apatchApkDir: currentApatchToolDir.value,
  apatchSuperKey: form.apatchSuperKey,
  kernelSuPath: form.kernelSuPath,
  kernelSuOptions: currentKernelToolOptions.value,
  kernelSuDir: currentKernelToolDir.value,
}));
const kernelSuState = computed(() => ({
  kernelSuKmi: form.kernelSuKmi,
  kernelSuKmiOptions: kernelSuKmiOptions.value,
  kernelSuDetectedKmi: kernelSuDetectedKmi.value,
}));
const rootState = computed(() => ({
  rootTargetPartition: rootTargetPartition.value,
  rootPatchToolLabel: rootPatchToolLabel.value,
}));

async function initializeBootPatchPage() {
  const previousDefaultOutputDir = String(defaultOutputDir.value || '').trim();
  const currentOutputDir = String(form.outputDir || '').trim();
  const shouldSyncOutputDir = !currentOutputDir || currentOutputDir === previousDefaultOutputDir;

  defaultOutputDir.value = await getSystemDownloadDir();
  if (shouldSyncOutputDir) {
    form.outputDir = defaultOutputDir.value;
  }

  await loadPatchToolOptions();
  await refreshKernelSuRuntime();
  await ensureApatchSuperKeyFilled(form.patchMode, { silent: true });
}

async function refresh() {
  if (patching.value || rooting.value) {
    ElMessage.warning('当前正在执行 Boot 修补流程，暂时不能刷新页面');
    return;
  }

  await initializeBootPatchPage();
}

defineExpose({ refresh });

watch(() => form.patchMode, (nextMode) => {
  if (
    ['magisk', 'magisk_alpha'].includes(normalizePatchMode(nextMode))
    && !getMagiskToolOptionsByMode(nextMode).some((item) => item.value === form.magiskApkPath)
  ) {
    form.magiskApkPath = getDefaultMagiskToolPath(nextMode);
  }

  if (isApatchMode(nextMode) && !getApatchToolOptionsByMode(nextMode).some((item) => item.value === form.apatchApkPath)) {
    form.apatchApkPath = getDefaultApatchToolPath(nextMode);
  }

  void ensureApatchSuperKeyFilled(nextMode);

  if (isKernelSuMode(nextMode) && !getKernelToolOptionsByMode(nextMode).some((item) => item.value === form.kernelSuPath)) {
    form.kernelSuPath = getDefaultKernelSuToolPath(nextMode);
  }
});

onMounted(async () => {
  await initializeBootPatchPage();

  unlistenLog = await listen('boot-patch-log', (event) => {
    handleBootPatchLog(event.payload);
  });
});

watch(() => form.kernelSuPath, () => {
  if (!isKernelSuMode(form.patchMode)) {
    return;
  }
  void refreshKernelSuRuntime();
});

watch(selectedSerial, (nextSerial) => {
  if (!nextSerial || !isKernelSuMode(form.patchMode)) {
    return;
  }
  if (skipNextKernelSuRuntimeAutoRefresh.value) {
    skipNextKernelSuRuntimeAutoRefresh.value = false;
    return;
  }
  if (patching.value || rooting.value) {
    return;
  }
  void refreshKernelSuRuntime({ silent: true });
});

onUnmounted(() => {
  if (unlistenLog) {
    unlistenLog();
  }
});
</script>

<style lang="scss" scoped>
.boot-patch-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
  // min-height: 100%;
  position: relative;
  // padding-top: 6px;
  isolation: isolate;
}

.boot-patch-page::before,
.boot-patch-page::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: -1;
}

.boot-patch-page::before {
  background: transparent;
}

.boot-patch-page::after {
  inset: 12px 0 auto;
  border-radius: 32px;
  background: transparent;
  opacity: 0;
}
</style>
