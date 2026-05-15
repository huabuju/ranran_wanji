<template>
  <div class="settings-page">
    <div class="settings-shell">
      <section
        v-for="group in settingGroups"
        :key="group.key"
        class="settings-group"
      >
        <el-card class="settings-card" shadow="never">
          <div
            v-for="item in group.items"
            :key="item.key"
            class="setting-row"
            :class="{ 'is-danger': item.danger }"
          >
            <div class="setting-copy">
              <h3 class="setting-title">{{ item.title }}</h3>
              <p v-if="item.description" class="setting-description">
                {{ item.description }}
              </p>
            </div>

            <div class="setting-action">
              <el-button
                round
                plain
                class="setting-button"
                :class="{ 'is-danger': item.danger }"
                :loading="item.key === 'cache' && cleaningCache"
                @click="item.action"
              >
                {{ item.actionText }}
              </el-button>
            </div>
          </div>
        </el-card>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useDeviceStore } from '@/utils/deviceStore';

const cleaningCache = ref(false);
const { stopPolling, startPolling } = useDeviceStore();

const settingGroups = [
  {
    key: 'resource-folders',
    items: [
      {
        key: 'driver-folder',
        title: '查看 / 安装驱动',
        description: '打开包含 ADB 与 Fastboot 驱动的本地文件夹',
        actionText: '打开文件夹',
        action: handleOpenDriverFolder,
      },
      {
        key: 'dependency-folder',
        title: '打开依赖包文件夹',
        description: '查看当前程序依赖目录，包含 platform-tools、scrcpy-core、aria2-core 等组件',
        actionText: '打开文件夹',
        action: handleOpenToolDependencyFolder,
      },
    ],
  },
  {
    key: 'tool-cache',
    items: [
      {
        key: 'cache',
        title: '工具缓存管理',
        description: '清除运行缓存与临时资源，不会删除你手动保存的下载文件或导出内容',
        actionText: '管理',
        action: handleClearToolCache,
        danger: true,
      },
    ],
  },
];

async function handleOpenDriverFolder() {
  try {
    await invoke('open_driver_folder');
  } catch (error) {
    console.error('Failed to open driver folder:', error);
    ElMessage.error(`打开失败: ${error}`);
  }
}

async function handleOpenToolDependencyFolder() {
  try {
    await invoke('open_tool_dependency_folder');
  } catch (error) {
    console.error('Failed to open tool dependency folder:', error);
    ElMessage.error(`打开失败: ${error}`);
  }
}

function clearBrowserCache() {
  localStorage.clear();
  sessionStorage.clear();
}

async function handleClearToolCache() {
  try {
    await ElMessageBox.confirm(
      '将立即清除本工具运行时依赖资源、临时文件等，请确保已保存所有重要数据和文件。',
      '确认清除工具缓存',
      {
        confirmButtonText: '确认清除',
        cancelButtonText: '取消',
        type: 'warning',
        distinguishCancelAndClose: true,
      }
    );
  } catch {
    return;
  }

  cleaningCache.value = true;
  try {
    stopPolling();
    clearBrowserCache();
    ElMessage.success('正在执行完整缓存清理，清理完成后应用会自动退出');
    await invoke('clear_tool_cache');
  } catch (error) {
    startPolling();
    console.error('Failed to clear tool cache:', error);
    ElMessage.error(`清理失败: ${error}`);
  } finally {
    cleaningCache.value = false;
  }
}

defineExpose({ refresh: () => {} });
</script>

<style lang="scss" scoped>
.settings-page {
  min-height: 100%;
  background: transparent;
}

.settings-shell {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-group {
  width: 100%;
}

.settings-card {
  border: 1px solid rgba(15, 23, 42, 0.06);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.94);
  box-shadow: 0 12px 30px rgba(15, 23, 42, 0.05);
  overflow: hidden;

  :deep(.el-card__body) {
    padding: 0;
  }
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  min-height: 72px;
  padding: 14px 18px;

  & + .setting-row {
    border-top: 1px solid rgba(15, 23, 42, 0.06);
  }
}

.setting-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.setting-title {
  margin: 0;
  font-size: 16px;
  line-height: 1.35;
  font-weight: 500;
  color: #161616;
  letter-spacing: -0.01em;
}

.setting-description {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
  color: #8a8f98;
  word-break: break-all;
}

.setting-action {
  flex-shrink: 0;
}

.setting-button {
  min-width: 124px;
  height: 38px;
  padding: 0 18px;
  border-radius: 999px;
  border-color: rgba(15, 23, 42, 0.16);
  background: rgba(255, 255, 255, 0.9);
  color: #2f3135;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;

  &:hover,
  &:focus-visible {
    border-color: rgba(15, 23, 42, 0.28);
    background: #fff;
    color: #111827;
    transform: translateY(-1px);
  }

  &.is-danger {
    border-color: rgba(220, 38, 38, 0.2);
    color: #b42318;

    &:hover,
    &:focus-visible {
      border-color: rgba(220, 38, 38, 0.36);
      color: #912018;
      background: rgba(255, 250, 250, 0.98);
    }
  }
}

.setting-row.is-danger .setting-title {
  color: #171717;
}

@media (max-width: 768px) {
  .settings-shell {
    gap: 12px;
  }

  .settings-card {
    border-radius: 10px;
  }

  .setting-row {
    min-height: auto;
    align-items: flex-start;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }

  .setting-action {
    width: 100%;
  }

  .setting-button {
    width: 100%;
    min-width: 0;
  }
}
</style>
