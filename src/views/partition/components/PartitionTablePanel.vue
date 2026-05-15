<template>
  <div class="left-table-panel page-table-card surface-card-strong">
    <div class="table-container page-table-inner" v-loading="loading">
      <el-table
        ref="tableRef"
        :data="data"
        class="custom-table full-width-table"
        height="100%"
        :row-class-name="tableRowClassName"
        @selection-change="$emit('selection-change', $event)"
      >
        <el-table-column type="selection" width="46" align="center" />
        <el-table-column prop="id" label="序号" width="120" align="center"></el-table-column>
        <el-table-column prop="name" label="名称" width="280">
          <template #default="{ row }"><span class="cell-name">{{ row.name }}</span></template>
        </el-table-column>
        <el-table-column prop="path" label="路径" show-overflow-tooltip>
          <template #default="{ row }"><span class="cell-path">{{ row.path }}</span></template>
        </el-table-column>
        <el-table-column label="操作" width="180" fixed="right" align="center">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button size="small" type="warning" plain :disabled="deviceMode !== 'fastboot' && deviceMode !== 'fastbootd'" @click="$emit('flash', row)">刷入</el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup>
defineProps({
  data: { type: Array, default: () => [] },
  loading: { type: Boolean, default: false },
  deviceMode: { type: String, default: 'unknown' },
  tableRowClassName: { type: Function, default: () => '' },
});

defineEmits(['selection-change', 'flash']);
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

  :deep(.custom-table) {
    --el-table-border-color: transparent;
    --el-table-header-bg-color: transparent;
    --el-table-header-text-color: var(--color-text-muted);
    font-size: 13px;
    background: transparent;

    .el-table__inner-wrapper::before { display: none; }
    .el-table__header-wrapper th {
      font-weight: 600;
      padding: 10px 0;
      border-bottom: 1px solid var(--color-divider);
      background: var(--table-header-bg);
    }
    .el-table__row {
      td {
        padding: 8px 0;
        border-bottom: 1px solid var(--color-divider);
      }
      &:hover > td { background-color: var(--table-row-hover); }
    }
  }
}

.cell-name { color: var(--text-code); font-weight: 600; }
.cell-path { color: var(--color-text-secondary); font-family: monospace; font-size: 12px; }

.action-buttons {
  display: flex;
  gap: 6px;
  justify-content: center;
}
</style>
