<template>
  <div class="filter-bar page-subtoolbar surface-card">
    <span class="filter-label">快速过滤：</span>
    <div class="filter-tags page-chip-group">
      <div v-for="filter in filters" :key="filter.value" class="filter-tag page-chip" :class="{ 'is-active': currentFilter === filter.value }" @click="$emit('change-filter', filter.value)">
        {{ filter.label }}
      </div>
    </div>

    <div class="batch-actions page-batch-actions">
      <template v-if="!isBatchExtracting">
        <el-button size="small" class="batch-btn extract-btn" :disabled="selectedCount === 0 || deviceMode !== 'device'" @click="$emit('batch-extract')">
          批量提取
          <span v-if="selectedCount > 0" class="badge">{{ selectedCount }}</span>
        </el-button>
      </template>
      <template v-else>
        <el-button size="small" class="batch-btn extract-btn running" disabled>提取中 {{ batchExtractProgress.current }}/{{ batchExtractProgress.total }}</el-button>
        <el-button size="small" class="batch-btn stop-btn" @click="$emit('cancel-extract')">停止</el-button>
      </template>

      <template v-if="!isBatchFormatting">
        <el-button size="small" class="batch-btn format-btn" :disabled="selectedCount === 0 || (deviceMode !== 'fastboot' && deviceMode !== 'fastbootd')" @click="$emit('batch-format')">
          批量格式化
          <span v-if="selectedCount > 0" class="badge">{{ selectedCount }}</span>
        </el-button>
      </template>
      <template v-else>
        <el-button size="small" class="batch-btn format-btn running" disabled>格式化中 {{ batchFormatProgress.current }}/{{ batchFormatProgress.total }}</el-button>
        <el-button size="small" class="batch-btn stop-btn" @click="$emit('cancel-format')">停止</el-button>
      </template>
    </div>

    <div class="fastboot-notice-inline">
      <span class="notice-text"><strong>刷入</strong>/<strong>格式化</strong>需在 <strong>Fastboot</strong> 模式下使用</span>
    </div>
  </div>
</template>

<script setup>
defineProps({
  filters: { type: Array, default: () => [] },
  currentFilter: { type: String, default: 'all' },
  selectedCount: { type: Number, default: 0 },
  deviceMode: { type: String, default: 'unknown' },
  isBatchExtracting: { type: Boolean, default: false },
  isBatchFormatting: { type: Boolean, default: false },
  batchExtractProgress: { type: Object, default: () => ({ current: 0, total: 0 }) },
  batchFormatProgress: { type: Object, default: () => ({ current: 0, total: 0 }) },
});

defineEmits(['change-filter', 'batch-extract', 'batch-format', 'cancel-extract', 'cancel-format']);
</script>

<style lang="scss" scoped>
.filter-label {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-right: 12px;
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

    &.format-btn:not(:disabled) {
      color: var(--color-danger);
      border-color: rgba(var(--color-danger-rgb), 0.22);
      background: rgba(var(--color-danger-rgb), 0.08);
      .badge { background: var(--color-danger); }
      &:hover { background: rgba(var(--color-danger-rgb), 0.14); border-color: rgba(var(--color-danger-rgb), 0.3); }
    }

    &:disabled { opacity: 0.4; cursor: not-allowed; }
    &.running { opacity: 1 !important; cursor: default; animation: pulse-border 1.4s ease-in-out infinite; }
    &.stop-btn {
      color: var(--color-warning);
      border-color: rgba(var(--color-warning-rgb), 0.24);
      background: rgba(var(--color-warning-rgb), 0.1);
      font-weight: 600;
    }
  }
}

.fastboot-notice-inline {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  background: linear-gradient(135deg, rgba(var(--color-warning-rgb), 0.14), rgba(var(--color-warning-rgb), 0.08));
  border: 1px solid rgba(var(--color-warning-rgb), 0.2);
  border-radius: 20px;
}

.notice-text {
  font-size: 12px;
  color: var(--text-warning);
  line-height: 1;
}

@keyframes pulse-border {
  0%, 100% { box-shadow: 0 0 0 0 rgba(var(--color-primary-rgb), 0.2); }
  50% { box-shadow: 0 0 0 3px rgba(var(--color-primary-rgb), 0); }
}
</style>
