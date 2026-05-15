<template>
  <footer class="status-bar flex-between">
    <div class="status-left flex-y-center">
      <template v-if="!isConnected">
        <span class="status-dot" :class="{ 'is-connected': isConnected }"></span>
        <span class="status-device">{{ displayDeviceName }}</span>
        <span class="status-sep">-</span>
        <span class="status-mode">{{ currentMode }}</span>
      </template>

      <template v-else>
        <span class="multi-label">设备切换:</span>
        <el-dropdown trigger="click" @command="handleSwitchDevice" placement="top" popper-class="device-dropdown">
          <div class="device-dropdown-trigger flex-y-center">
            <span class="status-dot" :class="getStateClass(selectedDevice?.state) || 'is-connected'"></span>
            <span class="status-device">{{ displayDeviceName }}</span>
            <svg class="dropdown-caret" viewBox="0 0 1024 1024" width="10" height="10" fill="currentColor">
              <path d="M512 320l320 384H192z"></path>
            </svg>
          </div>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="device in connectedDevices"
                :key="device.serial"
                :command="device.serial"
                :class="{ 'is-active': device.serial === selectedSerial }"
              >
                <div class="dropdown-device-item flex-y-center">
                  <span class="device-tag-dot" :class="getStateClass(device.state)"></span>
                  <div class="device-tag-text">
                    <span class="device-tag-name">{{ getDeviceDisplayName(device) }}</span>
                    <span v-if="shouldShowRawSerial(device)" class="device-tag-serial">{{ formatSerial(device.serial) }}</span>
                  </div>
                  <span class="device-tag-mode">{{ getModeShort(device.state) }}</span>
                </div>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <span class="status-sep">-</span>
        <span class="status-mode">{{ currentMode }}</span>
      </template>
    </div>

    <div class="status-right flex-y-center">
      <el-button type="info" link class="status-btn" @click="openCmd">
        <SmartIcon name="terminal" color="var(--color-info)" :size="10" />
        <span class="btn-text">CMD</span>
      </el-button>
      <el-button type="info" link class="status-btn" @click="openTaskManager">
        <SmartIcon name="monitor_pc" color="var(--color-success)" :size="10" />
        <span class="btn-text">设备管理器</span>
      </el-button>
      <div class="status-sep-v"></div>
      <el-button type="info" link class="status-btn theme-toggle" @click="cycleTheme">
        <SmartIcon :name="themeIcon" :color="themeColor" :size="10" />
        <span class="btn-text">{{ themeText }}</span>
      </el-button>
    </div>
  </footer>
</template>

<script setup>
import { computed, onMounted, onUnmounted } from 'vue';
import { openPlatformToolsCmd, openDeviceManager } from '@/api/device';
import SmartIcon from '@/components/common/SmartIcon.vue';
import { useDeviceStore } from '@/utils/deviceStore';
import { themeMode, cycleTheme } from '@/utils/themeStore';

const mode = themeMode;

const themeIcon = computed(() => {
  if (mode.value === 'auto') return 'monitor_pc';
  return mode.value === 'dark' ? 'moon' : 'sun';
});

const themeText = computed(() => {
  if (mode.value === 'auto') return '自动模式';
  return mode.value === 'dark' ? '深色模式' : '浅色模式';
});

const themeColor = computed(() => {
  if (mode.value === 'auto') return 'var(--color-primary)';
  return mode.value === 'dark' ? 'var(--brand-primary-strong)' : 'var(--brand-yellow)';
});

const {
  isConnected,
  displayDeviceName,
  currentMode,
  connectedDevices,
  getDeviceDisplayName,
  selectedSerial,
  selectedDevice,
  setSelectedDevice,
  startPolling,
  stopPolling,
} = useDeviceStore();

onMounted(() => {
  startPolling();
});

onUnmounted(() => {
  stopPolling();
});

function handleSwitchDevice(serial) {
  setSelectedDevice(serial);
}

function formatSerial(serial) {
  return serial.length > 16 ? `${serial.slice(0, 14)}...` : serial;
}

function shouldShowRawSerial(device) {
  return !!device?.serial && getDeviceDisplayName(device) !== device.serial;
}

function getModeShort(state) {
  if (state === 'fastbootd') return 'FBD';
  if (state === 'fastboot') return 'FB';
  if (state === 'recovery') return 'REC';
  if (state === 'sideload') return 'SDL';
  return 'ADB';
}

function getStateClass(state) {
  if (state === 'fastboot' || state === 'fastbootd') return 'is-fastboot';
  if (state === 'recovery') return 'is-recovery';
  return 'is-adb';
}

async function openCmd() {
  try {
    await openPlatformToolsCmd();
  } catch (e) {
    console.error('打开 CMD 失败:', e);
  }
}

async function openTaskManager() {
  try {
    await openDeviceManager();
  } catch (e) {
    console.error('打开设备管理器失败:', e);
  }
}
</script>

<style lang="scss" scoped>
.status-bar {
  height: var(--statusbar-height);
  background: var(--statusbar-bg);
  border-top: 1px solid var(--statusbar-border);
  padding: 0 16px;
  flex-shrink: 0;

  .status-left {
    gap: 6px;
    font-size: 12px;
    color: var(--color-text-secondary);
    min-width: 0;
    overflow: hidden;

    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--status-offline);
      flex-shrink: 0;
      transition: background-color 0.3s;

      &.is-connected {
        background: var(--status-online);
      }

      &.is-adb {
        background: var(--status-online);
      }

      &.is-fastboot {
        background: var(--status-fastboot);
      }

      &.is-recovery {
        background: var(--status-recovery);
      }
    }

    .status-device {
      font-weight: 500;
      color: var(--color-text-primary);
      max-width: 160px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .status-sep {
      color: var(--color-text-muted);
      flex-shrink: 0;
    }

    .multi-label {
      color: var(--color-text-muted);
      flex-shrink: 0;
      white-space: nowrap;
    }

    .device-dropdown-trigger {
      cursor: pointer;
      padding: 2px 6px;
      border-radius: 4px;
      transition: background-color 0.2s;
      gap: 6px;

      &:hover {
        background-color: var(--surface-soft);
      }

      .dropdown-caret {
        color: var(--color-text-muted);
        transition: transform 0.2s;
      }
    }
  }

  :deep(.device-dropdown) {
    .el-dropdown-menu__item {
      padding: 6px 12px;
      font-size: 12px;

      &.is-active {
        color: var(--color-primary);
        font-weight: 500;
        background-color: var(--color-primary-light);
      }
    }
  }

  .dropdown-device-item {
    gap: 8px;

    .device-tag-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      flex-shrink: 0;

      &.is-adb {
        background: var(--status-online);
      }

      &.is-fastboot {
        background: var(--status-fastboot);
      }

      &.is-recovery {
        background: var(--status-recovery);
      }
    }

    .device-tag-text {
      min-width: 0;
      display: flex;
      flex-direction: column;
      gap: 2px;
      flex: 1;
    }

    .device-tag-name {
      max-width: 160px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .device-tag-serial {
      max-width: 160px;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: 10px;
      color: var(--color-text-muted);
      font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
    }

    .device-tag-mode {
      font-size: 10px;
      opacity: 0.7;
      background: var(--surface-soft);
      padding: 0 4px;
      border-radius: 4px;
    }
  }

  .status-right {
    gap: 4px;

    .status-btn {
      font-size: 11.5px;
      color: var(--color-text-secondary);
      padding: 4px 8px;
      border-radius: var(--radius-sm);
      transition: all 0.15s;
      display: flex;
      align-items: center;
      gap: 8px;

      &:hover {
        background: var(--surface-soft) !important;
        color: var(--color-text-primary) !important;
      }
    }

    .status-sep-v {
      width: 1px;
      height: 14px;
      background: var(--statusbar-border);
      margin: 0 4px;
    }

    .theme-toggle {
      &:hover {
        background: var(--color-primary-light) !important;
      }
    }
  }

  .status-mode {
    font-size: 11px;
    color: var(--color-text-muted);
    text-transform: uppercase;
  }
}
</style>
