import { invoke } from '@tauri-apps/api/core';

export async function fetchXiaomiRomCatalog() {
  return await invoke('fetch_xiaomirom_catalog');
}

export async function fetchXiaomiRomModelRoms(seriesUrl) {
  return await invoke('fetch_xiaomirom_model_roms', { seriesUrl });
}

export async function resolveXiaomiRomDownloadUrls(pageUrl) {
  return await invoke('resolve_xiaomirom_download_urls', { pageUrl });
}
