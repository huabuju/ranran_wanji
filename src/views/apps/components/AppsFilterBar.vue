<template>
  <div class="filter-bar page-subtoolbar surface-card">
    <div class="left-filter page-filter-left">
      <div class="filter-tags page-chip-group">
        <div class="filter-tag page-chip" :class="{ 'is-active': appType === true }" @click="$emit('change-app-type', true)">用户应用</div>
        <div class="filter-tag page-chip" :class="{ 'is-active': appType === false }" @click="$emit('change-app-type', false)">系统应用</div>
      </div>
      <div class="divider"></div>
      <div class="filter-tags page-chip-group">
        <div class="filter-tag page-chip" :class="{ 'is-active': statusFilter === 'all' }" @click="$emit('change-status-filter', 'all')">全部</div>
        <div class="filter-tag page-chip" :class="{ 'is-active': statusFilter === 'running' }" @click="$emit('change-status-filter', 'running')">运行中</div>
        <div class="filter-tag page-chip" :class="{ 'is-active': statusFilter === 'frozen' }" @click="$emit('change-status-filter', 'frozen')">已冻结</div>
      </div>
    </div>

    <div class="right-batch-actions page-filter-right">
      <div class="batch-actions page-batch-actions">
        <template v-if="!isBatchExtracting">
          <el-button size="small" class="batch-btn extract-btn" :disabled="selectedCount === 0" @click="$emit('batch-extract')">
            批量提取
            <span v-if="selectedCount > 0" class="badge">{{ selectedCount }}</span>
          </el-button>
        </template>
        <template v-else>
          <el-button size="small" class="batch-btn extract-btn running" disabled>提取中 {{ batchExtractProgress.current }}/{{ batchExtractProgress.total }}</el-button>
          <el-button size="small" class="batch-btn stop-btn" @click="$emit('cancel-extract')">停止</el-button>
        </template>

        <template v-if="!isBatchUninstalling">
          <el-button size="small" class="batch-btn uninstall-btn" :disabled="selectedCount === 0" @click="$emit('batch-uninstall')">
            批量卸载
            <span v-if="selectedCount > 0" class="badge">{{ selectedCount }}</span>
          </el-button>
        </template>
        <template v-else>
          <el-button size="small" class="batch-btn uninstall-btn running" disabled>卸载中 {{ batchUninstallProgress.current }}/{{ batchUninstallProgress.total }}</el-button>
          <el-button size="small" class="batch-btn stop-btn" @click="$emit('cancel-uninstall')">停止</el-button>
        </template>

        <div class="divider"></div>

        <template v-if="!isBatchNetworkBlocking">
          <el-button size="small" class="batch-btn net-block-btn" :disabled="selectedCount === 0" @click="$emit('batch-block-network')">
            批量禁网
            <span v-if="selectedCount > 0" class="badge">{{ selectedCount }}</span>
          </el-button>
        </template>
        <template v-else>
          <el-button size="small" class="batch-btn net-block-btn running" disabled>禁网中 {{ batchNetworkBlockProgress.current }}/{{ batchNetworkBlockProgress.total }}</el-button>
          <el-button size="small" class="batch-btn stop-btn" @click="$emit('cancel-block-network')">停止</el-button>
        </template>

        <template v-if="!isBatchNetworkUnblocking">
          <el-button size="small" class="batch-btn net-unblock-btn" :disabled="selectedCount === 0" @click="$emit('batch-unblock-network')">
            批量联网
            <span v-if="selectedCount > 0" class="badge">{{ selectedCount }}</span>
          </el-button>
        </template>
        <template v-else>
          <el-button size="small" class="batch-btn net-unblock-btn running" disabled>联网中 {{ batchNetworkUnblockProgress.current }}/{{ batchNetworkUnblockProgress.total }}</el-button>
          <el-button size="small" class="batch-btn stop-btn" @click="$emit('cancel-unblock-network')">停止</el-button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  appType: { type: Boolean, default: true },
  statusFilter: { type: String, default: 'all' },
  selectedCount: { type: Number, default: 0 },
  isBatchExtracting: { type: Boolean, default: false },
  isBatchUninstalling: { type: Boolean, default: false },
  isBatchNetworkBlocking: { type: Boolean, default: false },
  isBatchNetworkUnblocking: { type: Boolean, default: false },
  batchExtractProgress: { type: Object, default: () => ({ current: 0, total: 0 }) },
  batchUninstallProgress: { type: Object, default: () => ({ current: 0, total: 0 }) },
  batchNetworkBlockProgress: { type: Object, default: () => ({ current: 0, total: 0 }) },
  batchNetworkUnblockProgress: { type: Object, default: () => ({ current: 0, total: 0 }) },
});

defineEmits([
  'change-app-type',
  'change-status-filter',
  'batch-extract',
  'batch-uninstall',
  'batch-block-network',
  'batch-unblock-network',
  'cancel-extract',
  'cancel-uninstall',
  'cancel-block-network',
  'cancel-unblock-network',
]);
</script>

<style lang="scss" scoped>
.filter-bar {
  font-size: 13px;

  .divider {
    width: 1px;
    height: 18px;
    background: var(--border-soft);
  }
}

.batch-actions {
  .batch-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 30px;
    padding: 0 12px;
    border-radius: var(--radius-full);
    font-size: 12px;
    border: 1px solid var(--border-soft);
    background: var(--surface-chip);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;

    .badge {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 16px;
      height: 16px;
      border-radius: 8px;
      font-size: 10px;
      font-weight: 700;
      line-height: 1;
      background: var(--color-text-muted);
      color: var(--text-on-primary);
      padding: 4px;
      margin-left: 4px;
      box-sizing: border-box;
    }

    &.extract-btn:not(:disabled) {
      color: var(--color-info);
      border-color: rgba(var(--color-info-rgb), 0.22);
      background: rgba(var(--color-info-rgb), 0.08);
      .badge { background: var(--color-info); }
      &:hover { background: rgba(var(--color-info-rgb), 0.14); border-color: rgba(var(--color-info-rgb), 0.3); }
    }

    &.uninstall-btn:not(:disabled) {
      color: var(--color-danger);
      border-color: rgba(var(--color-danger-rgb), 0.22);
      background: rgba(var(--color-danger-rgb), 0.08);
      .badge { background: var(--color-danger); }
      &:hover { background: rgba(var(--color-danger-rgb), 0.14); border-color: rgba(var(--color-danger-rgb), 0.3); }
    }

    &.net-block-btn:not(:disabled) {
      color: var(--color-primary);
      border-color: rgba(var(--color-primary-rgb), 0.22);
      background: rgba(var(--color-primary-rgb), 0.08);
      .badge { background: var(--color-primary); }
      &:hover { background: rgba(var(--color-primary-rgb), 0.14); border-color: rgba(var(--color-primary-rgb), 0.3); }
    }

    &.net-unblock-btn:not(:disabled) {
      color: var(--color-success);
      border-color: rgba(var(--color-success-rgb), 0.22);
      background: rgba(var(--color-success-rgb), 0.08);
      .badge { background: var(--color-success); }
      &:hover { background: rgba(var(--color-success-rgb), 0.14); border-color: rgba(var(--color-success-rgb), 0.3); }
    }

    &:disabled {
      opacity: 0.4;
      cursor: not-allowed;
    }

    &.running {
      opacity: 1 !important;
      cursor: default;
      animation: pulse-border 1.4s ease-in-out infinite;
    }

    &.stop-btn {
      color: var(--color-warning);
      border-color: rgba(var(--color-warning-rgb), 0.24);
      background: rgba(var(--color-warning-rgb), 0.1);
      font-weight: 600;
      &:hover { background: rgba(var(--color-warning-rgb), 0.16); border-color: rgba(var(--color-warning-rgb), 0.32); }
    }
  }

  :deep(.el-button) {
    border-radius: var(--radius-full);
    font-size: 12px;
    height: 30px;
    padding: 0 12px;
    margin-left: 0;
  }
}

@keyframes pulse-border {
  0%, 100% { box-shadow: 0 0 0 0 rgba(var(--color-primary-rgb), 0.2); }
  50% { box-shadow: 0 0 0 3px rgba(var(--color-primary-rgb), 0); }
}
</style>
