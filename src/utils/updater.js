import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { formatDateTime } from './date';
import { useUpdateStore } from './updateStore';

const { showUpdateDialog, isChecking, updateInfo } = useUpdateStore();

/**
 * 比较版本号
 * @param {string} remote 远程版本号 (例: 1.0.4)
 * @param {string} local 本地版本号 (例: 1.0.3)
 * @returns {boolean} 是否有新版本
 */
function isNewerVersion(remote, local) {
  // 移除开头的 'v' 并按 '.' 或 '-' 分割
  const parse = (v) => v.replace(/^v/i, '').split(/[.-]/).map(part => isNaN(part) ? part : Number(part));
  
  const remoteParts = parse(remote);
  const localParts = parse(local);

  for (let i = 0; i < Math.max(remoteParts.length, localParts.length); i++) {
    const r = remoteParts[i] !== undefined ? remoteParts[i] : 0;
    const l = localParts[i] !== undefined ? localParts[i] : 0;
    
    if (r > l) return true;
    if (r < l) return false;
  }
  return false;
}

function getLocalDateVersion() {
  if (typeof __APP_BUILD_TIME__ !== 'string' || !__APP_BUILD_TIME__) {
    return '';
  }

  return formatDateTime(__APP_BUILD_TIME__, 'YYYYMMDDHHmmss');
}

function normalizeDateVersion(value) {
  return String(value || '').replace(/\D/g, '');
}

function hasNewerDateVersion(remote, local) {
  const remoteDateVersion = normalizeDateVersion(remote);
  const localDateVersion = normalizeDateVersion(local);

  if (!remoteDateVersion || !localDateVersion) {
    return false;
  }

  return remoteDateVersion > localDateVersion;
}

/**
 * 检查更新并弹出提示
 * @returns {Promise<'updated' | 'no_update' | 'failed'>}
 */
export async function checkUpdate(options = {}) {
  const { silent = false } = options;

  if (!silent) {
    isChecking.value = true;
  }
  try {
    // 1. 获取本地版本
    const localVersion = await getVersion();
    const localDateVersion = getLocalDateVersion();

    // 2. 获取远程版本信息 (通过 Rust 绕过 CORS)
    const jsonStr = await invoke('get_online_update_json');
    const remoteData = JSON.parse(jsonStr);

    if (!remoteData || !remoteData.version) {
      console.warn('Update check: Invalid response format');
      return 'failed';
    }

    const remoteDateVersion = remoteData.dateVersion || remoteData.date;
    const hasUpdate = isNewerVersion(remoteData.version, localVersion)
      || (remoteData.version === localVersion && hasNewerDateVersion(remoteDateVersion, localDateVersion));

    // 3. 对比版本
    if (hasUpdate) {
      // 4. 更新状态并显示弹窗
      updateInfo.value = {
        version: remoteData.version,
        localVersion: localVersion,
        dateVersion: remoteDateVersion,
        localDateVersion,
        date: remoteData.date,
        notes: Array.isArray(remoteData.notes) ? remoteData.notes : [remoteData.notes],
        url: remoteData.url
      };

      showUpdateDialog.value = true;
      return 'updated';
    }
    return 'no_update';
  } catch (error) {
    console.error('Update check failed:', error);
    return 'failed';
  } finally {
    if (!silent) {
      isChecking.value = false;
    }
  }
}

/**
 * 用于“更新日志”手动查看
 */
export async function fetchUpdateInfo() {
  try {
    const jsonStr = await invoke('get_online_update_json');
    const remoteData = JSON.parse(jsonStr);
    
    if (remoteData) {
      updateInfo.value.changelog = remoteData.changelog || [];
      return remoteData;
    }
  } catch (error) {
    console.error('Failed to fetch update info:', error);
    throw error;
  }
}
