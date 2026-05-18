import { computed, ref } from 'vue';
import { getBootPatchToolOptions } from '@/api/bootPatch';
import {
  isApatchMode,
  isKernelSuMode,
  normalizePatchMode,
} from './bootPatchMode';

export function useBootPatchToolOptions(form) {
  const magiskApkOptions = ref([]);
  const magiskAlphaApkOptions = ref([]);
  const apatchApkOptions = ref([]);
  const kernelSuOptions = ref([]);
  const folkPatchApkOptions = ref([]);
  const kernelSuNextOptions = ref([]);
  const sukiSuUltraOptions = ref([]);
  const magiskApkDir = ref('');
  const magiskAlphaApkDir = ref('');
  const apatchApkDir = ref('');
  const kernelSuDir = ref('');
  const folkPatchApkDir = ref('');
  const kernelSuNextDir = ref('');
  const sukiSuUltraDir = ref('');

  function getApatchToolOptionsByMode(mode) {
    return normalizePatchMode(mode) === 'folkpatch' ? folkPatchApkOptions.value : apatchApkOptions.value;
  }

  function getApatchToolDirByMode(mode) {
    return normalizePatchMode(mode) === 'folkpatch' ? folkPatchApkDir.value : apatchApkDir.value;
  }

  function getMagiskToolOptionsByMode(mode) {
    return normalizePatchMode(mode) === 'magisk_alpha' ? magiskAlphaApkOptions.value : magiskApkOptions.value;
  }

  function getMagiskToolDirByMode(mode) {
    return normalizePatchMode(mode) === 'magisk_alpha' ? magiskAlphaApkDir.value : magiskApkDir.value;
  }

  function getKernelToolOptionsByMode(mode) {
    const normalized = normalizePatchMode(mode);
    if (normalized === 'kernelsu_next') return kernelSuNextOptions.value;
    if (normalized === 'sukisu_ultra') return sukiSuUltraOptions.value;
    return kernelSuOptions.value;
  }

  function getKernelToolDirByMode(mode) {
    const normalized = normalizePatchMode(mode);
    if (normalized === 'kernelsu_next') return kernelSuNextDir.value;
    if (normalized === 'sukisu_ultra') return sukiSuUltraDir.value;
    return kernelSuDir.value;
  }

  const currentApatchToolOptions = computed(() => getApatchToolOptionsByMode(form.patchMode));
  const currentApatchToolDir = computed(() => getApatchToolDirByMode(form.patchMode));
  const currentMagiskToolOptions = computed(() => getMagiskToolOptionsByMode(form.patchMode));
  const currentMagiskToolDir = computed(() => getMagiskToolDirByMode(form.patchMode));
  const currentKernelToolOptions = computed(() => getKernelToolOptionsByMode(form.patchMode));
  const currentKernelToolDir = computed(() => getKernelToolDirByMode(form.patchMode));

  function pickDefaultOptionValue(options, preferredNames = []) {
    if (!Array.isArray(options) || options.length === 0) {
      return '';
    }

    const preferredSet = new Set(preferredNames.map((item) => String(item).trim().toLowerCase()));
    const preferredOption = options.find((item) => preferredSet.has(String(item.label || '').trim().toLowerCase()));
    return preferredOption?.value || options[0].value || '';
  }

  function getDefaultMagiskToolPath(mode = form.patchMode) {
    return pickDefaultOptionValue(getMagiskToolOptionsByMode(mode));
  }

  function getDefaultApatchToolPath(mode = form.patchMode) {
    return pickDefaultOptionValue(getApatchToolOptionsByMode(mode));
  }

  function getDefaultKernelSuToolPath(mode = form.patchMode) {
    return pickDefaultOptionValue(getKernelToolOptionsByMode(mode));
  }

  function getEffectivePatchToolPath(mode) {
    if (isKernelSuMode(mode)) {
      return form.kernelSuPath || getDefaultKernelSuToolPath(mode);
    }

    if (isApatchMode(mode)) {
      return form.apatchApkPath || getDefaultApatchToolPath(mode);
    }

    return form.magiskApkPath || getDefaultMagiskToolPath(mode);
  }

  function getInitialPatchMode() {
    if (kernelSuOptions.value.length > 0) {
      return 'kernelsu';
    }
    if (kernelSuNextOptions.value.length > 0) {
      return 'kernelsu_next';
    }
    if (sukiSuUltraOptions.value.length > 0) {
      return 'sukisu_ultra';
    }
    if (apatchApkOptions.value.length > 0) {
      return 'apatch';
    }
    if (folkPatchApkOptions.value.length > 0) {
      return 'folkpatch';
    }
    if (magiskApkOptions.value.length > 0) {
      return 'magisk';
    }
    if (magiskAlphaApkOptions.value.length > 0) {
      return 'magisk_alpha';
    }
    return 'kernelsu';
  }

  async function loadPatchToolOptions() {
    let toolOptions;
    try {
      toolOptions = await getBootPatchToolOptions();
    } catch {
      toolOptions = {
        magiskApkOptions: [],
        magiskAlphaApkOptions: [],
        apatchApkOptions: [],
        folkPatchApkOptions: [],
        kernelSuOptions: [],
        kernelSuNextOptions: [],
        sukiSuUltraOptions: [],
        magiskApkDir: '',
        magiskAlphaApkDir: '',
        apatchApkDir: '',
        folkPatchApkDir: '',
        kernelSuDir: '',
        kernelSuNextDir: '',
        sukiSuUltraDir: '',
      };
    }

    const magiskOptions = Array.isArray(toolOptions?.magiskApkOptions) ? toolOptions.magiskApkOptions : [];
    const magiskAlphaOptions = Array.isArray(toolOptions?.magiskAlphaApkOptions) ? toolOptions.magiskAlphaApkOptions : [];
    const apatchOptions = Array.isArray(toolOptions?.apatchApkOptions) ? toolOptions.apatchApkOptions : [];
    const folkPatchOptions = Array.isArray(toolOptions?.folkPatchApkOptions) ? toolOptions.folkPatchApkOptions : [];
    const kernelsuOptions = Array.isArray(toolOptions?.kernelSuOptions) ? toolOptions.kernelSuOptions : [];
    const kernelSuNextModeOptions = Array.isArray(toolOptions?.kernelSuNextOptions) ? toolOptions.kernelSuNextOptions : [];
    const sukiSuUltraModeOptions = Array.isArray(toolOptions?.sukiSuUltraOptions) ? toolOptions.sukiSuUltraOptions : [];

    magiskApkOptions.value = magiskOptions;
    magiskAlphaApkOptions.value = magiskAlphaOptions;
    apatchApkOptions.value = apatchOptions;
    folkPatchApkOptions.value = folkPatchOptions;
    kernelSuOptions.value = kernelsuOptions;
    kernelSuNextOptions.value = kernelSuNextModeOptions;
    sukiSuUltraOptions.value = sukiSuUltraModeOptions;
    magiskApkDir.value = String(toolOptions?.magiskApkDir || '').trim();
    magiskAlphaApkDir.value = String(toolOptions?.magiskAlphaApkDir || '').trim();
    apatchApkDir.value = String(toolOptions?.apatchApkDir || '').trim();
    folkPatchApkDir.value = String(toolOptions?.folkPatchApkDir || '').trim();
    kernelSuDir.value = String(toolOptions?.kernelSuDir || '').trim();
    kernelSuNextDir.value = String(toolOptions?.kernelSuNextDir || '').trim();
    sukiSuUltraDir.value = String(toolOptions?.sukiSuUltraDir || '').trim();

    if (!getMagiskToolOptionsByMode(form.patchMode).some((item) => item.value === form.magiskApkPath)) {
      form.magiskApkPath = getDefaultMagiskToolPath(form.patchMode);
    }

    if (!getKernelToolOptionsByMode(form.patchMode).some((item) => item.value === form.kernelSuPath)) {
      form.kernelSuPath = getDefaultKernelSuToolPath(form.patchMode);
    }

    if (!getApatchToolOptionsByMode(form.patchMode).some((item) => item.value === form.apatchApkPath)) {
      form.apatchApkPath = getDefaultApatchToolPath(form.patchMode);
    }

    if (
      ['magisk', 'magisk_alpha'].includes(normalizePatchMode(form.patchMode))
      && !getMagiskToolOptionsByMode(form.patchMode).length
    ) {
      form.patchMode = getInitialPatchMode();
    }
  }

  return {
    magiskApkOptions,
    magiskAlphaApkOptions,
    apatchApkOptions,
    kernelSuOptions,
    folkPatchApkOptions,
    kernelSuNextOptions,
    sukiSuUltraOptions,
    magiskApkDir,
    magiskAlphaApkDir,
    apatchApkDir,
    kernelSuDir,
    folkPatchApkDir,
    kernelSuNextDir,
    sukiSuUltraDir,
    currentApatchToolOptions,
    currentApatchToolDir,
    currentMagiskToolOptions,
    currentMagiskToolDir,
    currentKernelToolOptions,
    currentKernelToolDir,
    getApatchToolOptionsByMode,
    getApatchToolDirByMode,
    getMagiskToolOptionsByMode,
    getMagiskToolDirByMode,
    getKernelToolOptionsByMode,
    getKernelToolDirByMode,
    pickDefaultOptionValue,
    getDefaultMagiskToolPath,
    getDefaultApatchToolPath,
    getDefaultKernelSuToolPath,
    getEffectivePatchToolPath,
    getInitialPatchMode,
    loadPatchToolOptions,
  };
}
