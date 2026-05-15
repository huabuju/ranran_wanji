import { invoke } from '@tauri-apps/api/core';

export async function fetchHyperOsFansCatalog() {
  return await invoke('fetch_hyperos_catalog');
}

export async function fetchHyperOsFansModelRoms(codename) {
  return await invoke('fetch_hyperos_model_roms', { codename });
}
