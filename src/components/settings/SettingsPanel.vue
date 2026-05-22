<template>
  <div class="settings-page">
    <div class="settings-shell">
      <div class="settings-card">
        <div
          v-for="item in settingItems"
          :key="item.key"
          class="setting-row"
          :class="{ 'is-danger': item.danger }"
        >
          <SmartIcon
            class="setting-icon"
            :name="item.icon"
            :color="item.color"
            :size="24"
          />

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
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SmartIcon from '@/components/common/SmartIcon.vue';
import { useDeviceStore } from '@/utils/deviceStore';

const cleaningCache = ref(false);
const { stopPolling, startPolling } = useDeviceStore();

const settingItems = [
  {
    key: 'dependency-folder',
    title: '打开资源包文件夹',
    description: '查看当前程序依赖目录，包含 platform-tools、scrcpy-core、aria2-core 等组件',
    actionText: '打开文件夹',
    icon: 'folder',
    color: '#8b5cf6',
    action: handleOpenToolDependencyFolder,
  },
  {
    key: 'cache',
    title: '工具缓存管理',
    description: '清除运行缓存与临时资源，不会删除你手动保存的下载文件或导出内容',
    actionText: '管理',
    icon: 'trash',
    color: '#ff6b6b',
    action: handleClearToolCache,
    danger: true,
  },
];

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
}

.settings-card {
  border: 1px solid var(--border-soft);
  border-radius: 18px;
  background: transparent;
  box-shadow: none;
  overflow: hidden;
}

.setting-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 14px;
  min-height: 74px;
  padding: 13px 22px;
  background: transparent;

  & + .setting-row {
    border-top: 1px solid var(--color-divider);
  }
}

.setting-icon {
  flex-shrink: 0;
  border-radius: 14px;
}

.setting-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.setting-title {
  margin: 0;
  font-size: 13px;
  line-height: 1.3;
  // font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.01em;
}

.setting-description {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.75;
  color: var(--color-text-secondary);
  word-break: break-word;
}

.setting-action {
  display: flex;
  flex-shrink: 0;
  justify-content: flex-end;
}

.setting-button {
  min-width: 92px;
  height: 30px;
  padding: 0 14px;
  border-radius: 999px;
  border-color: var(--border-soft);
  background: var(--surface-chip);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 600;
  transition: all 0.2s ease;

  &:hover,
  &:focus-visible {
    border-color: rgba(var(--color-primary-rgb), 0.38);
    background: var(--surface-chip-hover);
    color: var(--color-primary);
    transform: translateY(-1px);
  }

  &.is-danger {
    border-color: rgba(var(--color-danger-rgb), 0.28);
    background: var(--danger-soft);
    color: var(--text-danger-strong);

    &:hover,
    &:focus-visible {
      border-color: rgba(var(--color-danger-rgb), 0.46);
      background: var(--danger-soft);
      color: var(--text-danger-strong);
    }
  }
}

@media (max-width: 768px) {
  .settings-card {
    border-radius: 16px;
  }

  .setting-row {
    grid-template-columns: auto minmax(0, 1fr);
    gap: 12px;
    min-height: auto;
    padding: 13px 14px;
  }

  .setting-action {
    grid-column: 2;
    justify-content: flex-start;
  }

  .setting-button {
    min-width: 0;
  }
}
</style>
