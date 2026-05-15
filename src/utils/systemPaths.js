import { downloadDir } from '@tauri-apps/api/path';

export async function getSystemDownloadDir() {
  try {
    return await downloadDir();
  } catch (error) {
    console.error('Failed to resolve system download dir:', error);
    return '';
  }
}
