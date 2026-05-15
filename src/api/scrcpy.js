import { invoke } from '@tauri-apps/api/core';
import { useDeviceStore } from '@/utils/deviceStore';

/**
 * 启动 Scrcpy
 * @param {Array<string>} args - 传递给 scrcpy.exe 的参数数组
 * @returns {Promise<void>}
 */
export async function startScrcpy(args = []) {
  const { selectedSerial } = useDeviceStore();
  const serial = selectedSerial.value || null;
  try {
    await invoke('start_scrcpy', { serial, args });
  } catch (error) {
    console.error('启动 Scrcpy 失败:', error);
    throw error;
  }
}

/**
 * 停止 Scrcpy
 * @returns {Promise<void>}
 */
export async function stopScrcpy() {
  try {
    await invoke('stop_scrcpy');
  } catch (error) {
    console.error('停止 Scrcpy 失败:', error);
    throw error;
  }
}
