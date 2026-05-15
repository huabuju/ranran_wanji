import { invoke } from '@tauri-apps/api/core';

export async function fetchGithubApkAssets(repo, token = '') {
  return await invoke('fetch_github_apk_assets', {
    request: { repo, token },
  });
}
