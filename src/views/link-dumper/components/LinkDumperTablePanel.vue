<template>
  <div class="left-table-panel page-table-card surface-card-strong" v-loading="fetching">
    <div class="table-container page-table-inner">
      <el-table
        ref="partitionTable"
        :data="partitions"
        height="100%"
        class="custom-table full-width-table"
        @selection-change="$emit('selection-change', $event)"
      >
        <el-table-column type="selection" width="55" :selectable="checkSelectable" />
        <el-table-column label="#" width="60" align="center">
          <template #default="{ $index }"><span class="text-xs text-gray-400">{{ $index + 1 }}</span></template>
        </el-table-column>
        <el-table-column property="partition_name" label="分区名称" align="left" min-width="146">
          <template #default="{ row }">
            <div class="cell-name partition-name-cell">
              <span class="partition-name-text">{{ getPartitionName(row.partition_name) }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="大小" width="220" align="center" sortable :sort-method="sortBySize">
          <template #default="{ row }"><span class="partition-size-text">{{ row.size_readable }}</span></template>
        </el-table-column>
        <el-table-column label="哈希值" align="left" min-width="160">
          <template #default="{ row }">
            <el-tooltip :content="row.hash" effect="light" class="hash-tooltip" open-delay="600">
              <span class="partition-hash-text overflow-hidden">{{ row.hash }}</span>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="200" align="center">
          <template #default="{ row }">
            <el-tag v-if="extractedList.includes(row.partition_name)" size="small" type="success">已提取</el-tag>
            <el-tag v-else size="small" type="info">未提取</el-tag>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup>
defineProps({
  fetching: { type: Boolean, default: false },
  partitions: { type: Array, default: () => [] },
  extractedList: { type: Array, default: () => [] },
  checkSelectable: { type: Function, required: true },
  getPartitionName: { type: Function, required: true },
  sortBySize: { type: Function, required: true },
});

defineEmits(['selection-change']);
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

  .cell-name { font-weight: 600; color: var(--text-code); font-size: 13px; }
}

.full-width-table { width: 100%; }
.partition-name-cell { display: flex; align-items: center; }
.partition-name-text { font-size: 13px; font-weight: 700; color: var(--color-text-primary); }
.partition-size-text { font-size: 12px; color: var(--color-text-secondary); }
.hash-tooltip { width: 80%; }
.partition-hash-text { font-size: 11px; color: var(--color-text-muted); font-family: 'JetBrains Mono', 'Consolas', monospace; }
.overflow-hidden { width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
