import { invoke } from '@tauri-apps/api/core';

export async function fetchXfuCatalog() {
  return await invoke('fetch_xfu_catalog');
}

export async function fetchXfuModelRoms(codename) {
  return await invoke('fetch_xfu_model_roms', { codename });
}
