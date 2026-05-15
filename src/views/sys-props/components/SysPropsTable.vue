<template>
  <div class="main-content dashboard-card page-table-card surface-card-strong" v-loading="loading">
    <el-auto-resizer>
      <template #default="{ height, width }">
        <el-table-v2 :columns="columns" :data="data" :width="width" :height="height" fixed class="props-table-v2">
          <template #cell="{ rowData, column }">
            <template v-if="column.key === 'key'">
              <el-tooltip :content="rowData.key" placement="top" :show-after="500">
                <span class="prop-key truncate-text">{{ rowData.key }}</span>
              </el-tooltip>
            </template>

            <template v-else-if="column.key === 'value'">
              <el-tooltip :content="rowData.value || '(空)'" placement="top" :show-after="500">
                <span class="prop-value truncate-text" :class="{ 'is-empty': !rowData.value }">
                  {{ rowData.value || '(空)' }}
                </span>
              </el-tooltip>
            </template>

            <template v-else-if="column.key === 'operations'">
              <div class="row-actions">
                <el-tooltip content="复制" placement="top">
                  <button class="icon-action-btn copy" @click="$emit('copy', rowData)">
                    <SmartIcon name="copy" :size="14" color="var(--color-success)" :show-background="false" />
                  </button>
                </el-tooltip>
                <el-tooltip content="编辑" placement="top">
                  <button class="icon-action-btn edit" @click="$emit('edit', rowData)">
                    <SmartIcon name="edit" :size="14" color="var(--color-info)" :show-background="false" />
                  </button>
                </el-tooltip>
                <el-tooltip :content="currentTab === 'getprop' ? '清空' : '删除'" placement="top">
                  <button class="icon-action-btn delete" @click="$emit('delete', rowData)">
                    <SmartIcon name="trash" :size="14" color="var(--color-danger)" :show-background="false" />
                  </button>
                </el-tooltip>
              </div>
            </template>
          </template>
        </el-table-v2>
      </template>
    </el-auto-resizer>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  columns: { type: Array, default: () => [] },
  data: { type: Array, default: () => [] },
  loading: { type: Boolean, default: false },
  currentTab: { type: String, default: 'getprop' },
});

defineEmits(['copy', 'edit', 'delete']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.dashboard-card {
  min-width: 0;
}

.main-content {
  flex: 1;
  overflow: hidden;
  min-height: 0;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);

  :deep(.props-table-v2) {
    background: transparent;
    border: none;

    .el-table-v2__row-cell {
      border-bottom: 1px solid var(--color-divider);
      background: transparent;
      padding: 0 12px;
      display: flex;
      align-items: center;
    }

    .el-table-v2__header-cell {
      background: var(--table-header-bg);
      font-weight: 600;
      color: var(--color-text-secondary);
      font-size: 12px;
      border-bottom: 1px solid var(--color-divider);
      padding: 0 12px;
      display: flex;
      align-items: center;
    }

    .el-table-v2__row:hover .el-table-v2__row-cell {
      background-color: var(--table-row-hover);
    }
  }
}

.prop-key {
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 13px;
  color: var(--text-code);
  font-weight: 600;
}

.prop-value {
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 13px;
  color: var(--color-text-secondary);

  &.is-empty {
    color: var(--color-text-muted);
    font-style: italic;
    opacity: 0.7;
  }
}

.truncate-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.row-actions {
  display: flex;
  justify-content: center;
  gap: 8px;
}

.icon-action-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border);
  background: var(--surface-soft);
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    transform: translateY(-1px);
    box-shadow: 0 8px 16px -14px rgba(15, 23, 42, 0.28);
  }

  &.copy:hover {
    border-color: rgba(var(--color-success-rgb), 0.3);
    background: rgba(var(--color-success-rgb), 0.08);
  }

  &.edit:hover {
    border-color: rgba(var(--color-info-rgb), 0.3);
    background: rgba(var(--color-info-rgb), 0.08);
  }

  &.delete:hover {
    border-color: rgba(var(--color-danger-rgb), 0.3);
    background: rgba(var(--color-danger-rgb), 0.08);
  }
}
</style>
