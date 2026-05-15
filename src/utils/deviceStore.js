import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { fetchConnectedDevices } from '@/api/device';

// 控制是否记录并展示“设备自动轮询日志”。
// 关闭后不会移除悬浮日志面板，只是不再向其中写入自动扫描日志。
export const ENABLE_DEVICE_SCAN_LOGGING = false;

const connectedDevices = ref([]);
const selectedSerial = ref(localStorage.getItem('device_selected_serial') || '');
const deviceLabels = ref({});
const scanLogs = ref([]);

const pendingLabelLoads = new Set();
const deviceMissCounts = new Map();
const scanLoggingScopes = new Set();

const DEVICE_POLL_INTERVAL_MS = 600;
const DEVICE_OFFLINE_GRACE_POLLS = 2;
const DEVICE_SCAN_LOG_LIMIT = 400;

let pollTimer = null;
let pollingEnabled = false;
let pollingSessionId = 0;
let scanSequence = 0;
let lastScanSignature = '';

const isConnected = computed(() => connectedDevices.value.length > 0);

const selectedDevice = computed(() => {
  if (connectedDevices.value.length === 0) return null;
  return connectedDevices.value.find(device => device.serial === selectedSerial.value) || connectedDevices.value[0];
});

const displayDeviceName = computed(() => (
  selectedDevice.value ? getDeviceDisplayName(selectedDevice.value) : '未连接设备'
));

const currentMode = computed(() => {
  if (!selectedDevice.value) return '未连接设备';

  const state = selectedDevice.value.state;
  if (state === 'fastbootd') return 'FastbootD 模式';
  if (state === 'fastboot') return 'Fastboot 模式';
  if (state === 'recovery') return 'Recovery 模式';
  if (state === 'sideload') return 'Sideload 模式';
  return '系统模式 (ADB)';
});

function getLogTime() {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false });
}

function appendScanLog(content, type = 'info', tag = 'SCAN') {
  if (!ENABLE_DEVICE_SCAN_LOGGING) return;
  if (scanLoggingScopes.size === 0) return;

  scanLogs.value.push({
    time: getLogTime(),
    content,
    type,
    tag,
  });

  if (scanLogs.value.length > DEVICE_SCAN_LOG_LIMIT) {
    scanLogs.value = scanLogs.value.slice(-DEVICE_SCAN_LOG_LIMIT);
  }
}

function clearScanLogs() {
  scanLogs.value = [];
  lastScanSignature = '';
}

function startScanLogging(scope = 'overview') {
  if (!ENABLE_DEVICE_SCAN_LOGGING) {
    clearScanLogs();
    return;
  }

  const wasDisabled = scanLoggingScopes.size === 0;
  scanLoggingScopes.add(scope);

  if (wasDisabled) {
    appendScanLog('已开启设备自动扫描日志，等待下一次轮询...', 'info', 'LOGGER');
  }
}

function stopScanLogging(scope = 'overview') {
  scanLoggingScopes.delete(scope);
}

function isAdbState(state) {
  return ['device', 'recovery', 'sideload', 'unauthorized'].includes(state);
}

function getDeviceSource(device) {
  if (device?.source) return device.source;
  return isAdbState(device?.state) ? 'adb' : 'fastboot';
}

function formatDeviceLog(device) {
  return `${getDeviceSource(device).toUpperCase()} | serial=${device.serial} | state=${device.state}`;
}

function buildDeviceSignature(devices) {
  return [...devices]
    .map(device => `${getDeviceSource(device)}:${device.serial}:${device.state}`)
    .sort()
    .join('|');
}

function formatRawOutput(output) {
  const normalized = String(output || '').trim();
  return normalized || '<empty>';
}

function getSnapshotDevices(snapshot) {
  return Array.isArray(snapshot?.devices) ? snapshot.devices : [];
}

function appendScanChannelLog(scanId, tag, report, verbose = false) {
  const devices = Array.isArray(report?.devices) ? report.devices : [];
  const summary = devices.length > 0
    ? devices.map(formatDeviceLog).join(' ; ')
    : '未发现设备';

  appendScanLog(
    `轮询 #${scanId} ${tag} 扫描结果：${devices.length} 台，${summary}`,
    devices.length > 0 ? 'success' : 'info',
    tag,
  );

  if (!verbose && !report?.raw_output) {
    return;
  }

  appendScanLog(`轮询 #${scanId} ${tag} 原始输出：\n${formatRawOutput(report?.raw_output)}`, 'info', tag);
}

function appendScanSnapshotLog(scanId, snapshot, stableDevices, reconcileMeta) {
  const rawDevices = getSnapshotDevices(snapshot);
  const duration = Number(snapshot?.duration_ms || 0);
  const signature = [
    buildDeviceSignature(rawDevices),
    buildDeviceSignature(stableDevices || []),
    formatRawOutput(snapshot?.adb_report?.raw_output),
    formatRawOutput(snapshot?.fastboot_report?.raw_output),
  ].join('||');

  const verbose = (
    signature !== lastScanSignature
    || rawDevices.length === 0
    || stableDevices.length === 0
    || reconcileMeta.keptByGrace.length > 0
    || reconcileMeta.removedAfterGrace.length > 0
  );

  appendScanLog(
    `轮询 #${scanId} 完成：ADB=${snapshot?.adb_report?.devices?.length || 0}，Fastboot=${snapshot?.fastboot_report?.devices?.length || 0}，原始合并=${rawDevices.length}，稳定结果=${stableDevices.length}，耗时=${duration}ms`,
    stableDevices.length > 0 ? 'success' : 'warning',
    'SCAN',
  );

  appendScanChannelLog(scanId, 'ADB', snapshot?.adb_report, verbose);
  appendScanChannelLog(scanId, 'FASTBOOT', snapshot?.fastboot_report, verbose);

  for (const item of reconcileMeta.keptByGrace) {
    appendScanLog(
      `轮询 #${scanId} 离线宽限保留：serial=${item.serial} | state=${item.state} | miss=${item.missCount}/${DEVICE_OFFLINE_GRACE_POLLS}`,
      'warning',
      'STORE',
    );
  }

  for (const item of reconcileMeta.removedAfterGrace) {
    appendScanLog(
      `轮询 #${scanId} 离线宽限结束：serial=${item.serial} | state=${item.state} | miss=${item.missCount}/${DEVICE_OFFLINE_GRACE_POLLS}`,
      'warning',
      'STORE',
    );
  }

  lastScanSignature = signature;
}

function setSelectedDevice(serial) {
  selectedSerial.value = serial;
  localStorage.setItem('device_selected_serial', serial);
}

function getCompactSerialName(serial) {
  const value = String(serial || '').trim();
  if (!value) return '未连接设备';

  const wirelessMatch = value.match(/^adb-([A-Za-z0-9]+)-[^.]+(?:\._adb-tls-connect\._tcp.*)?$/);
  if (wirelessMatch?.[1]) {
    return wirelessMatch[1];
  }

  return value;
}

function normalizeDeviceLabel(deviceName, serial) {
  const name = String(deviceName || '').trim();
  if (!name || name === '--') {
    return getCompactSerialName(serial);
  }
  return name;
}

function getDeviceDisplayName(deviceOrSerial) {
  const serial = typeof deviceOrSerial === 'string'
    ? deviceOrSerial
    : deviceOrSerial?.serial;

  if (!serial) return '未连接设备';
  return deviceLabels.value[serial] || getCompactSerialName(serial);
}

function pruneDeviceLabels(devices) {
  const activeSerials = new Set(devices.map(device => device.serial));
  const nextLabels = Object.fromEntries(
    Object.entries(deviceLabels.value).filter(([serial]) => activeSerials.has(serial)),
  );

  if (Object.keys(nextLabels).length !== Object.keys(deviceLabels.value).length) {
    deviceLabels.value = nextLabels;
  }
}

function reconcileConnectedDevices(devices) {
  const nextDevices = [...devices];
  const nextSerials = new Set(devices.map(device => device.serial));
  const keptByGrace = [];
  const removedAfterGrace = [];

  for (const device of devices) {
    deviceMissCounts.set(device.serial, 0);
  }

  for (const previousDevice of connectedDevices.value) {
    const serial = previousDevice?.serial;
    if (!serial || nextSerials.has(serial)) continue;

    const missCount = (deviceMissCounts.get(serial) || 0) + 1;
    if (missCount >= DEVICE_OFFLINE_GRACE_POLLS) {
      deviceMissCounts.delete(serial);
      removedAfterGrace.push({ serial, state: previousDevice.state, missCount });
      continue;
    }

    deviceMissCounts.set(serial, missCount);
    nextDevices.push(previousDevice);
    keptByGrace.push({ serial, state: previousDevice.state, missCount });
  }

  for (const serial of Array.from(deviceMissCounts.keys())) {
    if (!nextDevices.some(device => device.serial === serial)) {
      deviceMissCounts.delete(serial);
    }
  }

  return {
    devices: nextDevices,
    keptByGrace,
    removedAfterGrace,
  };
}

async function ensureDeviceLabel(device) {
  if (!device || device.state !== 'device') return;

  const serial = String(device.serial || '').trim();
  if (!serial || pendingLabelLoads.has(serial)) return;

  const cachedLabel = deviceLabels.value[serial];
  if (cachedLabel && cachedLabel !== getCompactSerialName(serial)) return;

  pendingLabelLoads.add(serial);
  try {
    const info = await invoke('get_device_info', { serial });
    const label = normalizeDeviceLabel(info?.device_name, serial);
    if (label !== deviceLabels.value[serial]) {
      deviceLabels.value = { ...deviceLabels.value, [serial]: label };
    }
  } catch (error) {
    console.debug('Skip resolving device label:', serial, error);
  } finally {
    pendingLabelLoads.delete(serial);
  }
}

async function checkDeviceConnection() {
  const scanId = ++scanSequence;

  try {
    const snapshot = await fetchConnectedDevices();
    const rawDevices = getSnapshotDevices(snapshot);
    const previousSelectedSerial = selectedSerial.value;
    const reconcileMeta = reconcileConnectedDevices(rawDevices);
    const stableDevices = reconcileMeta.devices;

    if (scanLoggingScopes.size > 0) {
      appendScanSnapshotLog(scanId, snapshot, stableDevices, reconcileMeta);
    }

    connectedDevices.value = stableDevices;
    pruneDeviceLabels(stableDevices);

    if (stableDevices.length === 0) {
      const hadSelection = Boolean(previousSelectedSerial);
      selectedSerial.value = '';
      deviceLabels.value = {};
      deviceMissCounts.clear();
      localStorage.removeItem('device_selected_serial');

      if (hadSelection) {
        appendScanLog(`轮询 #${scanId} 稳定列表为空，已清空当前选中设备`, 'warning', 'STORE');
      }
      return;
    }

    const stillExists = stableDevices.some(device => device.serial === selectedSerial.value);
    if (!stillExists) {
      appendScanLog(
        `轮询 #${scanId} 当前选中设备丢失，自动切换到 ${stableDevices[0].serial}`,
        previousSelectedSerial ? 'warning' : 'info',
        'STORE',
      );
      setSelectedDevice(stableDevices[0].serial);
    }

    for (const device of stableDevices) {
      void ensureDeviceLabel(device);
    }
  } catch (error) {
    appendScanLog(
      `轮询 #${scanId} 扫描异常：${error instanceof Error ? error.message : String(error)}`,
      'error',
      'SCAN',
    );
    console.error('Failed to fetch devices:', error);
  }
}

function startPolling(intervalMs = DEVICE_POLL_INTERVAL_MS) {
  if (pollingEnabled) return;

  pollingEnabled = true;
  const sessionId = ++pollingSessionId;

  const poll = async () => {
    if (!pollingEnabled || sessionId !== pollingSessionId) return;
    pollTimer = null;

    try {
      await checkDeviceConnection();
    } finally {
      if (!pollingEnabled || sessionId !== pollingSessionId) return;
      pollTimer = setTimeout(poll, intervalMs);
    }
  };

  void poll();
}

function stopPolling() {
  pollingEnabled = false;
  pollingSessionId += 1;

  if (!pollTimer) return;
  clearTimeout(pollTimer);
  pollTimer = null;
}

export function useDeviceStore() {
  return {
    connectedDevices,
    scanLogs,
    selectedSerial,
    selectedDevice,
    isConnected,
    displayDeviceName,
    getDeviceDisplayName,
    currentMode,
    clearScanLogs,
    setSelectedDevice,
    startScanLogging,
    startPolling,
    stopScanLogging,
    stopPolling,
    checkDeviceConnection,
  };
}
