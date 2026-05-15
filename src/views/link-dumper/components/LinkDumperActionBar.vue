<template>
  <div class="action-bar page-subtoolbar surface-card">
    <div class="left-actions page-filter-left">
      <el-button type="success" :loading="fetching" :disabled="extracting" @click="$emit('fetch')">获取分区列表</el-button>
      <el-button type="primary" :disabled="selectedCount === 0 || extracting" @click="$emit('extract-selected')">提取选中 ({{ selectedCount }})</el-button>
      <el-button type="warning" :disabled="total === 0 || extracting" @click="$emit('extract-all')">全选提取</el-button>
    </div>
    <div class="divider ml-4 mr-4"></div>
    <div class="output-info">
      <span class="text-xs text-gray-500 mr-2">输出目录:</span>
      <el-tag size="small" type="info" class="output-dir-tag">{{ outputDir || '未选择' }}</el-tag>
      <el-button link type="primary" size="small" :disabled="extracting" class="change-output-btn" @click="$emit('select-output')">更改</el-button>
    </div>
  </div>
</template>

<script setup>
defineProps({
  fetching: { type: Boolean, default: false },
  extracting: { type: Boolean, default: false },
  selectedCount: { type: Number, default: 0 },
  total: { type: Number, default: 0 },
  outputDir: { type: String, default: '' },
});

defineEmits(['fetch', 'extract-selected', 'extract-all', 'select-output']);
</script>

<style lang="scss" scoped>
.action-bar {
  font-size: 13px;

  .divider { width: 1px; height: 16px; background: var(--border-soft); margin: 0 20px; }
}

.output-info {
  display: flex;
  align-items: center;
}

.output-dir-tag {
  margin-left: 8px;
}

.change-output-btn {
  margin-left: 4px;
}
</style>
