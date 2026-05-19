import { ref } from 'vue';
import { getKernelSuRuntime } from '@/api/bootPatch';
import { isKernelSuMode } from './bootPatchMode';

export function useKernelSuRuntime({ form, patching, rooting, addLocalLog }) {
  const kernelSuRuntimeLoading = ref(false);
  const kernelSuKmiOptions = ref([]);
  const kernelSuDetectedKmi = ref('');
  const kernelSuSuggestedKmi = ref('');
  let kernelSuRuntimeRequestId = 0;

  async function refreshKernelSuRuntime(options = {}) {
    const silent = options.silent === true;
    const currentPath = String(form.kernelSuPath || '').trim();
    const previousSuggestedKmi = kernelSuSuggestedKmi.value;
    const requestId = ++kernelSuRuntimeRequestId;
    const shouldFreezeRuntimeState = () => patching.value || rooting.value;
    const shouldMuteRuntimeFeedback = () => silent || shouldFreezeRuntimeState();

    if (!isKernelSuMode(form.patchMode)) {
      kernelSuRuntimeLoading.value = false;
      kernelSuKmiOptions.value = [];
      kernelSuDetectedKmi.value = '';
      kernelSuSuggestedKmi.value = '';
      return;
    }

    if (!currentPath) {
      kernelSuRuntimeLoading.value = false;
      kernelSuKmiOptions.value = [];
      kernelSuDetectedKmi.value = '';
      kernelSuSuggestedKmi.value = '';
      if (!form.kernelSuKmi || form.kernelSuKmi === previousSuggestedKmi) {
        form.kernelSuKmi = '';
      }
      return;
    }

    kernelSuRuntimeLoading.value = true;

    try {
      const runtime = await getKernelSuRuntime({ kernelSuPath: currentPath, patchMode: form.patchMode });
      if (requestId !== kernelSuRuntimeRequestId || shouldFreezeRuntimeState()) {
        return;
      }

      const supportedKmis = (runtime.supportedKmis || []).map((item) => String(item || '').trim()).filter(Boolean);

      kernelSuKmiOptions.value = supportedKmis.map((item) => ({
        label: item,
        value: item,
      }));
      kernelSuDetectedKmi.value = String(runtime.detectedKmi || '').trim();

      const preferredSuggestedKmi = String(runtime.defaultKmi || runtime.detectedKmi || '').trim();
      const nextSuggestedKmi = supportedKmis.includes(preferredSuggestedKmi) ? preferredSuggestedKmi : '';
      const shouldSyncKmi = !form.kernelSuKmi || form.kernelSuKmi === previousSuggestedKmi;

      kernelSuSuggestedKmi.value = nextSuggestedKmi;
      if (shouldSyncKmi) {
        form.kernelSuKmi = nextSuggestedKmi;
      }

      if (shouldMuteRuntimeFeedback()) {
        return;
      }

      if (nextSuggestedKmi) {
        addLocalLog(`已匹配到可用 KMI，并自动选择：${nextSuggestedKmi}`, 'info', 'UI');
      } else if (preferredSuggestedKmi) {
        addLocalLog(`检测到 KMI“${preferredSuggestedKmi}”但它不在当前 KernelSU 版本支持列表中，已取消自动填入，请手动从下拉列表选择。`, 'warning', 'UI');
      } else {
        addLocalLog('当前未检测到可自动匹配的 KMI，请手动从下拉列表选择。', 'warning', 'UI');
      }
    } catch (error) {
      if (requestId !== kernelSuRuntimeRequestId || shouldFreezeRuntimeState()) {
        return;
      }

      kernelSuKmiOptions.value = [];
      kernelSuDetectedKmi.value = '';
      kernelSuSuggestedKmi.value = '';
      if (!form.kernelSuKmi || form.kernelSuKmi === previousSuggestedKmi) {
        form.kernelSuKmi = '';
      }
      if (!shouldMuteRuntimeFeedback()) {
        addLocalLog(`读取 KernelSU KMI 列表失败：${error}`, 'warning', 'UI');
      }
    } finally {
      if (requestId === kernelSuRuntimeRequestId) {
        kernelSuRuntimeLoading.value = false;
      }
    }
  }

  return {
    kernelSuRuntimeLoading,
    kernelSuKmiOptions,
    kernelSuDetectedKmi,
    kernelSuSuggestedKmi,
    refreshKernelSuRuntime,
  };
}
