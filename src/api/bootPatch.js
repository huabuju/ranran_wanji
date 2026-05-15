import { invoke } from '@tauri-apps/api/core';
import { useDeviceStore } from '@/utils/deviceStore';

function getSerial() {
  const { selectedSerial } = useDeviceStore();
  return selectedSerial.value || null;
}

export async function patchBootImage(request) {
  return await invoke('patch_boot_image', {
    serial: getSerial(),
    request,
  });
}

export async function generateApatchSuperKey() {
  return await invoke('generate_apatch_super_key');
}

export async function getBootPatchToolOptions() {
  return await invoke('get_boot_patch_tool_options');
}

export async function getKernelSuRuntime(request) {
  return await invoke('get_kernelsu_runtime', {
    serial: getSerial(),
    request,
  });
}

export async function getKernelSuVersions() {
  return await invoke('get_kernelsu_versions');
}

export async function bootPatchOneKeyRoot(request) {
  return await invoke('boot_patch_one_key_root', {
    serial: getSerial(),
    request,
  });
}

export async function prepareBootPatchAutoSource() {
  return await invoke('prepare_boot_patch_auto_source', {
    serial: getSerial(),
  });
}
