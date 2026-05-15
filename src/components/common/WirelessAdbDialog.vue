<template>
  <el-dialog
    v-model="visible"
    title="无线调试 (Wireless ADB)"
    width="40%"
    class="wireless-adb-dialog app-dialog-shell"
    :close-on-click-modal="false"
    append-to-body
    destroy-on-close
  >
    <div 
      class="dialog-content"
      v-loading="isLoading"
      element-loading-text="ADB 进程处理中..."
      element-loading-background="var(--surface-overlay)"
    >
      <div class="header-icon-zone border-glass">
        <div class="icon-circle">
          <SmartIcon name="wifi" :size="20" color="var(--color-info)" :show-background="false" :show-decoration="false" />
        </div>
        <div class="header-copy">
          <p class="subtitle">通过同一 Wi-Fi 网络快速连接设备</p>
          <p class="subhint">如为首次连接，需您主动“配对”一次，后续可直接“连接”</p>
          <p class="subhint">找不到可配对设备？请确保已打开“无线调试-使用配对码配对”弹窗</p>
        </div>
      </div>

      <div class="mode-options">
        <div class="mode-switcher">
          <el-radio-group v-model="activeMode" size="default" class="custom-radio-group">
            <el-radio-button label="connect">连接</el-radio-button>
            <el-radio-button label="pair">配对</el-radio-button>
          </el-radio-group>
        </div>
        <div class="manual-toggle flex-y-center">
          <span class="toggle-label">手动模式</span>
          <el-switch v-model="isManual" size="small" />
        </div>
      </div>

      <div class="form-zone">
        <div class="ip-port-group">
          <div class="form-item ip-item">
            <label>IP 地址</label>
            <el-input 
              v-model="ipAddress" 
              :placeholder="isManual ? '例如 192.168.1.5' : '自动填充中...'" 
              :disabled="!isManual" 
              class="custom-input" 
            />
          </div>
          <div class="port-separator">:</div>
          <div class="form-item port-item">
            <label>{{ activeMode === 'connect' ? '端口' : '配对端口' }}</label>
            <el-input 
              v-model="port" 
              :placeholder="isManual ? '5555' : '自动填充'" 
              :disabled="!isManual" 
              class="custom-input" 
            />
          </div>
        </div>

        <div class="form-item pairing-code-item animate-in" v-if="activeMode === 'pair'">
          <label>配对码</label>
          <el-input v-model="pairingCode" placeholder="例如 123456" maxlength="6" class="custom-input" />
        </div>

        <div class="form-item quick-pick-item animate-in" v-if="!isManual">
          <div class="quick-pick-head">
            <label>附近可用设备</label>
            <span class="scan-count" v-if="filteredScannedDevices.length > 0">{{ filteredScannedDevices.length }} 台</span>
          </div>
          <div class="quick-pick-row">
            <el-select
              v-model="selectedScannedDevice"
              clearable
              filterable
              placeholder="自动扫描中，请耐心等待..."
              class="device-select"
              popper-class="wireless-adb-device-popper"
              @change="handleScannedDeviceChange"
            >
              <el-option
                v-for="device in filteredScannedDevices"
                :key="device.ip + device.port"
                :label="formatScannedDeviceCompactLabel(device)"
                :value="device.ip + ':' + device.port"
              >
                <div class="device-option">
                  <div class="device-option-main">
                    <!-- <span class="device-option-name">{{ device.instance_name }}</span> -->
                    <span class="device-option-addr">{{ device.ip }}:{{ device.port }}</span>
                  </div>
                  <span class="device-option-tag">{{ getServiceTypeTag(device) }}</span>
                </div>
              </el-option>
            </el-select>

          </div>
        </div>
      </div>

      <div class="info-alert" v-if="recentLog">
        <span class="log-text" :class="{ 'is-error': recentLog.isError }">{{ recentLog.message }}</span>
      </div>

      <div class="action-footer">
        <div class="left-actions">
          <el-button type="danger" plain @click="handleDisconnect" size="default" :loading="disconnecting">
            断开全部
          </el-button>
        </div>
        <el-button 
          type="primary" 
          @click="activeMode === 'connect' ? handleConnect() : handlePair()" 
          size="default" 
          class="connect-btn shadow-btn" 
          :loading="activeMode === 'connect' ? connecting : pairing"
        >
          {{ activeMode === 'connect' ? '立即连接' : '立即配对' }}
        </el-button>
      </div>
    </div>
  </el-dialog>

  <FloatingLog :logs="resolvedFloatingLogs" @clear="handleClearFloatLogs" v-if="props.showFloatLog"/>
</template>

<script setup>
import { computed, ref, watch, onUnmounted } from 'vue';
import { runAdb, fetchConnectedDevices, scanAdbDevices, adbPair } from '@/api/device';
import SmartIcon from '@/components/common/SmartIcon.vue';
import FloatingLog from '@/components/common/FloatingLog.vue';
const emit = defineEmits(['clear-float-logs']);

const visible = ref(false);
const ipAddress = ref('');
const port = ref('');
const pairingCode = ref('');
const activeMode = ref('connect'); // 'connect' | 'pair'
const isManual = ref(false);

const connecting = ref(false);
const pairing = ref(false);
const disconnecting = ref(false);
const recentLog = ref(null);
const detailLogs = ref([]);
const scannedDevices = ref([]);
const selectedScannedDevice = ref('');

let scanTimer = null;

const stopAutoScan = () => {
  if (scanTimer !== null) {
    clearTimeout(scanTimer);
    scanTimer = null;
  }
};

const startAutoScan = () => {
  stopAutoScan();
  if (isManual.value) return;

  const tick = async () => {
    if (isManual.value) {
      stopAutoScan();
      return;
    }
    try {
      const results = await scanAdbDevices();
      scannedDevices.value = results;
      const stillValid = filteredScannedDevices.value.some(
        device => `${device.ip}:${device.port}` === selectedScannedDevice.value
      );
      if (!stillValid) selectedScannedDevice.value = '';

      if (results.length > 0) {
        const matchedResults = filteredScannedDevices.value;

        if (matchedResults.length > 0) {
          const currentTarget = `${(ipAddress.value || '').trim()}:${(port.value || '').trim()}`;
          const matchedDevice = matchedResults.find(device => `${device.ip}:${device.port}` === currentTarget) || matchedResults[0];
          if (matchedDevice) {
            selectDevice(matchedDevice, true);
          }
        }
      }
    } catch {
      // 静默失败
    } finally {
      if (visible.value) {
        scanTimer = setTimeout(tick, 100);
      }
    }
  };
  tick();
};

onUnmounted(() => stopAutoScan());

const props = defineProps({
  showFloatLog: {
    type: Boolean,
    default: false
  },
  floatLogs: {
    type: Array,
    default: null
  }
});

const isLoading = computed(() => connecting.value || disconnecting.value);

const floatingLogs = computed(() =>
  detailLogs.value.map((item) => ({
    time: item.time,
    tag: item.step,
    content: item.message,
    type: item.isError ? 'error' : 'info'
  }))
);

const resolvedFloatingLogs = computed(() =>
  props.floatLogs ?? floatingLogs.value
);

const filteredScannedDevices = computed(() => {
  const targetType = activeMode.value === 'pair' ? '_adb-tls-pairing._tcp' : '_adb-tls-connect._tcp';
  return scannedDevices.value.filter(device => String(device.service_type || '').includes(targetType));
});

const open = () => {
  visible.value = true;
  recentLog.value = null;
  scannedDevices.value = [];
  selectedScannedDevice.value = '';
  activeMode.value = 'connect';
  pairingCode.value = '';
  isManual.value = true;

  startAutoScan();
};

watch(isManual, (val) => {
  if (val) {
    stopAutoScan();
    selectedScannedDevice.value = '';
  } else {
    startAutoScan();
  }
});

watch(visible, (val) => {
  if (!val) stopAutoScan();
});

const normalizeMessage = (message) => String(message ?? '').replace(/\r\n/g, '\n').trim();

const appendDetailLog = (step, message, isError = false) => {
  const now = new Date();
  detailLogs.value.push({
    time: now.toLocaleTimeString('zh-CN', { hour12: false }),
    step,
    message: normalizeMessage(message),
    isError
  });

  if (detailLogs.value.length > 300) {
    detailLogs.value.splice(0, detailLogs.value.length - 300);
  }
};

const setLog = (message, isError = false, step = '系统') => {
  recentLog.value = { message, isError };
  appendDetailLog(step, message, isError);
};

const clearDetailLogs = () => {
  detailLogs.value = [];
  recentLog.value = { message: '日志已清空', isError: false };
};

const handleClearFloatLogs = () => {
  if (props.floatLogs !== null) {
    emit('clear-float-logs');
    return;
  }

  clearDetailLogs();
};

const getTargetAddress = () => {
  const ip = ipAddress.value.trim();
  const p = port.value.trim() || '5555';
  if (!ip) throw new Error('请输入设备的 IP 地址');
  return `${ip}:${p}`;
};

const isConnectResponseSuccessful = (message) => {
  const text = normalizeMessage(message).toLowerCase();
  return text.includes('connected to') || text.includes('already connected to');
};

const formatScannedDeviceCompactLabel = (device) =>
  `${device.ip} : ${device.port}`;

const getServiceTypeTag = (device) => {
  const serviceType = String(device?.service_type || '');
  if (serviceType.includes('_adb-tls-pairing._tcp')) return 'Pair';
  if (serviceType.includes('_adb-tls-connect._tcp')) return 'Connect';
  return serviceType.includes('tls') ? 'TLS' : 'TCP';
};

watch(activeMode, () => {
  const currentTarget = `${(ipAddress.value || '').trim()}:${(port.value || '').trim()}`;
  const matchedDevice = filteredScannedDevices.value.find(device => `${device.ip}:${device.port}` === currentTarget);

  if (matchedDevice) {
    selectedScannedDevice.value = currentTarget;
    return;
  }

  selectedScannedDevice.value = '';
});

const handleConnect = async () => {
  try {
    const target = getTargetAddress();
    connecting.value = true;
    setLog(`正在尝试连接到 ${target} ...`, false, '连接');
    const resp = await runAdb(['connect', target], { useCurrentSerial: false });
    appendDetailLog('ADB 响应', resp);
    
    const connectAccepted = isConnectResponseSuccessful(resp);

    if (connectAccepted) {
      setLog(' 成功连接到设备！验证状态：已在线。', false, '连接');
    } else {
      setLog(' 连接失败：设备未出现在在线列表中，请确认 IP 和端口正确。', true, '连接');
    }
  } catch (e) {
    setLog(` 连接异常: ${e}`, true, '连接');
  } finally {
    connecting.value = false;
  }
};

const handlePair = async () => {
  try {
    const ip = (ipAddress.value || '').trim();
    const p = (port.value || '').trim();
    const code = (pairingCode.value || '').trim();

    pairing.value = true;
    const target = `${ip}:${p}`;
    setLog(`正在尝试配对到 ${target} ...`, false, '配对');

    const resp = await adbPair(target, code);
    appendDetailLog('配对响应', resp);

    if (resp.toLowerCase().includes('successfully paired')) {
      setLog('配对成功！请选择当前设备展示的端口，进行连接操作...', false, '配对');
      // 切换模式
      activeMode.value = 'connect';
      pairingCode.value = '';
    } else {
      setLog(`配对结果: ${resp}`, true, '配对');
    }
  } catch (e) {
    setLog(` 配对异常: ${e.message || e}`, true, '配对');
  } finally {
    pairing.value = false;
  }
};

const selectDevice = (device, isSilent = false) => {
  const targetId = `${device.ip}:${device.port}`;
  if (selectedScannedDevice.value === targetId && ipAddress.value === device.ip && port.value === device.port) return;

  ipAddress.value = device.ip;
  port.value = device.port;
  selectedScannedDevice.value = targetId;
  if (!isSilent) {
    appendDetailLog('扫描', `已选择发现的设备: ${device.instance_name} (${targetId})`);
  }
};

const handleScannedDeviceChange = (value) => {
  if (!value) return;
  const device = filteredScannedDevices.value.find(item => `${item.ip}:${item.port}` === value);
  if (device) {
    selectDevice(device);
  }
};

const handleDisconnect = async () => {
  try {
    disconnecting.value = true;
    setLog('正在断开所有无线连接...', false, '断开');

    const resp = await runAdb(['disconnect'], { useCurrentSerial: false });
    appendDetailLog('断开响应', resp);
    setLog(` 已断开: ${resp}`, false, '断开');
  } catch (e) {
    setLog(` 操作异常: ${e}`, true, '断开');
  } finally {
    disconnecting.value = false;
  }
};

defineExpose({ open });
</script>

<style lang="scss" scoped>
.dialog-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 0 8px 8px;
}

.header-icon-zone {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: linear-gradient(145deg, rgba(var(--color-primary-rgb), 0.08) 0%, var(--surface-soft) 100%);
  border-radius: 12px;
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-elevated-soft);

  .icon-circle {
    width: 42px;
    height: 42px;
    background: var(--surface-strong);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 6px 14px rgba(var(--color-primary-rgb), 0.14);
    flex-shrink: 0;
  }

  .header-copy {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .subtitle {
    font-size: 13px;
    color: var(--color-text-primary);
    margin: 0;
    text-align: left;
    font-weight: 600;
  }

  .subhint {
    font-size: 11px;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.45;
  }
}

.form-zone {
  display: flex;
  flex-direction: column;
  gap: 12px;

  .form-item {
    display: flex;
    flex-direction: column;
    gap: 6px;

    label {
      font-size: 12px;
      font-weight: 600;
      color: var(--color-text-primary);
      margin-left: 2px;
      line-height: 1.2;
    }
  }

  .ip-port-group {
    display: flex;
    align-items: flex-end;
    gap: 8px;

    .ip-item {
      flex: 3;
    }

    .port-item {
      flex: 1.5;
    }

    .port-separator {
      font-size: 16px;
      font-weight: bold;
      color: var(--color-text-muted);
      padding-bottom: 7px;
    }
  }

  .quick-pick-item {
    gap: 8px;
  }

  .quick-pick-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .scan-count {
    font-size: 11px;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .quick-pick-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .device-select {
    flex: 1;
  }


  .custom-input {
    :deep(.el-input__wrapper),
    :deep(.el-select__wrapper) {
      border-radius: 8px;
      box-shadow: 0 0 0 1px var(--color-border) inset;
      padding: 4px 12px;
      transition: all 0.2s;
      background-color: var(--surface-strong);

      &:hover,
      &.is-focus {
        box-shadow: 0 0 0 1px var(--color-primary) inset;
        background-color: var(--surface-soft);
      }
    }
  }
}

.mode-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px;
}

.manual-toggle {
  gap: 10px;
  background: var(--surface-soft);
  padding: 4px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-soft);
  height: 32px;

  .toggle-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }
}

.mode-switcher {
  display: flex;
  justify-content: center;

  :deep(.custom-radio-group) {
    .el-radio-button__inner {
      border-radius: 6px;
      padding: 7px 14px;
      font-weight: 600;
      font-size: 12px;
      border: 1px solid var(--color-border);
      box-shadow: none !important;
      transition: all 0.2s;
    }

    .el-radio-button:first-child .el-radio-button__inner {
      border-radius: 8px 0 0 8px;
    }
    .el-radio-button:last-child .el-radio-button__inner {
      border-radius: 0 8px 8px 0;
    }

    .el-radio-button__original-radio:checked + .el-radio-button__inner {
      background-color: var(--color-primary);
      border-color: var(--color-primary);
      color: var(--text-on-primary);
    }
  }
}

.animate-in {
  animation: slideDown 0.3s ease-out;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.info-alert {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 9px 12px;
  background: rgba(var(--color-success-rgb), 0.08);
  border: 1px solid rgba(var(--color-success-rgb), 0.22);
  border-radius: 8px;
  font-size: 12px;

  .log-text {
    flex: 1;
    color: var(--color-success);
    line-height: 1.5;
    word-break: break-all;

    &.is-error {
      color: var(--color-danger);
    }
  }

  &:has(.is-error) {
    background: rgba(var(--color-danger-rgb), 0.08);
    border-color: rgba(var(--color-danger-rgb), 0.22);
  }
}

.action-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 2px;

  .left-actions {
    display: flex;
    gap: 8px;
  }

  .connect-btn {
    border-radius: 8px;
    padding: 9px 20px;
    font-weight: 600;
    transition: transform 0.2s, box-shadow 0.2s;

    &:hover {
      transform: translateY(-1px);
      box-shadow: 0 8px 18px rgba(var(--color-primary-rgb), 0.28);
    }
  }
}

.bottom-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin-top: 6px;
  padding-top: 10px;
  border-top: 1px dashed var(--color-border);
  font-size: 11px;
  color: var(--color-text-muted);
  line-height: 1.4;
  text-align: center;

  .hint-icon {
    font-size: 13px;
    color: var(--color-border);
  }
}

</style>

<style lang="scss">
.wireless-adb-device-popper {
  .device-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-width: 0;
    width: 100%;
    padding: 2px 0;
  }

  .device-option-main {
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 8px;
    flex: 1;
  }

  .device-option-name {
    min-width: 0;
    max-width: 170px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    color: var(--color-text-primary);
    font-weight: 600;
  }

  .device-option-addr {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--color-text-secondary);
    font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  }

  .device-option-tag {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    min-width: 52px;
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(var(--color-primary-rgb), 0.1);
    color: var(--color-primary);
    font-size: 11px;
    font-weight: 700;
    line-height: 1.4;
  }
}
</style>
