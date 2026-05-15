import { invoke } from '@tauri-apps/api/core';

export async function fetchMiuierCatalog() {
  return await invoke('fetch_miuier_catalog');
}

export async function fetchMiuierModelRoms(codename) {
  return await invoke('fetch_miuier_model_roms', { codename });
}
