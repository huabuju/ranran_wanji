<template>
  <div class="left-table-panel page-table-card surface-card-strong">
    <div class="table-container page-table-inner">
      <el-auto-resizer>
        <template #default="{ height, width }">
          <el-table-v2
            ref="tableRef"
            :columns="columns"
            :data="data"
            :width="width"
            :height="height"
            :row-event-handlers="rowEventHandlers"
            fixed
            class="custom-table-v2"
          >
            <template #cell="{ rowData: row, column }">
              <template v-if="column.key === 'name'">
                <div class="cell-name flex items-center truncate-text">
                  <span class="app-name-text">{{ appLabels[row]?.label || row }}</span>
                </div>
              </template>

              <template v-else-if="column.key === 'package'">
                <span class="app-meta-text truncate-text">{{ row }}</span>
              </template>

              <template v-else-if="column.key === 'version'">
                <span class="app-meta-text truncate-text">{{ appLabels[row]?.version_name || '-' }}</span>
              </template>

              <template v-else-if="column.key === 'uid'">
                <span class="uid-badge">{{ appLabels[row]?.uid !== undefined ? appLabels[row]?.uid : '-' }}</span>
              </template>

              <template v-else-if="column.key === 'path'">
                <div class="path-cell truncate-text">
                  <el-tooltip :content="appLabels[row]?.source_dir || '-'" placement="top" :show-after="300">
                    <span class="path-text">{{ appLabels[row]?.source_dir || '-' }}</span>
                  </el-tooltip>
                  <span v-if="appLabels[row]?.source_dir" class="copy-icon" title="复制路径" @click.stop="$emit('copy-path', appLabels[row].source_dir)">⧉</span>
                </div>
              </template>

              <template v-else-if="column.key === 'actions'">
                <div class="action-cell">
                  <el-button
                    v-if="pkgDetails[row]?.is_enabled !== false"
                    size="small"
                    type="warning"
                    plain
                    @click.stop="$emit('toggle-freeze', row, 'freeze')"
                  >冻结</el-button>
                  <el-button
                    v-else
                    size="small"
                    type="primary"
                    plain
                    @click.stop="$emit('toggle-freeze', row, 'unfreeze')"
                  >解冻</el-button>

                  <el-dropdown size="small" trigger="click" @command="(cmd) => $emit('row-action', row, cmd)" @click.stop class="more-actions-dropdown">
                    <el-button size="small" type="info" plain @click.stop>
                      更多
                      <el-icon class="el-icon--right"><ArrowDown /></el-icon>
                    </el-button>
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item command="stop">强行停止</el-dropdown-item>
                        <el-dropdown-item command="clear" class="warning-item">清除数据</el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </template>
            </template>

            <template #empty>
              <div class="empty-state">
                <SmartIcon name="info" :size="32" color="var(--color-text-muted)" class="mb-2" />
                <span>{{ loading ? '查询中...' : '暂无数据' }}</span>
              </div>
            </template>
          </el-table-v2>
        </template>
      </el-auto-resizer>
    </div>
  </div>
</template>

<script setup>
import { ArrowDown } from '@element-plus/icons-vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  columns: { type: Array, default: () => [] },
  data: { type: Array, default: () => [] },
  loading: { type: Boolean, default: false },
  appLabels: { type: Object, default: () => ({}) },
  pkgDetails: { type: Object, default: () => ({}) },
  rowEventHandlers: { type: Object, default: () => ({}) },
});

defineEmits(['copy-path', 'toggle-freeze', 'row-action']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.left-table-panel {
  flex: 1;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}

.table-container {
  flex: 1;
  overflow: hidden;
  height: 100%;
  position: relative;

  :deep(.custom-table-v2) {
    background: transparent;
    border: none;

    .el-table-v2__header-cell {
      background: var(--table-header-bg);
      color: var(--color-text-secondary);
      font-size: 12px;
      font-weight: 600;
      border-bottom: 1px solid var(--color-divider);
      padding: 0 12px;
      display: flex;
      align-items: center;
    }

    .el-table-v2__row-cell {
      border-bottom: 1px solid var(--color-divider);
      background: transparent;
      display: flex;
      align-items: center;
      padding: 0 12px;
    }

    .el-table-v2__row:hover .el-table-v2__row-cell {
      background-color: var(--table-row-hover);
    }

    .truncate-text {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      flex: 1;
      min-width: 0;
    }
  }
}

.app-name-text {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.app-meta-text {
  font-size: 11px;
  color: var(--color-text-secondary);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}

.uid-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  background: var(--surface-soft);
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--color-text-muted);
}

.path-cell {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;

  .path-text {
    flex: 1;
    font-size: 11px;
    font-family: monospace;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }

  .copy-icon {
    flex-shrink: 0;
    font-size: 14px;
    color: var(--color-text-muted);
    cursor: pointer;
    line-height: 1;
    transition: color 0.2s;
    user-select: none;
    &:hover { color: var(--color-primary); }
  }
}

.action-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  flex-wrap: nowrap;
}

.more-actions-dropdown {
  margin-left: 10px;
}

:deep(.warning-item) {
  color: var(--color-warning) !important;
}
</style>
