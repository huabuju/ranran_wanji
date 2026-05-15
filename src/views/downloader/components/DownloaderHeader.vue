<template>
  <div class="dl-header top-card page-toolbar surface-card">
    <div class="header-left page-toolbar-left">
      <div class="dl-tabs">
        <div class="dl-tab-item" :class="{ active: activeTab === 'downloading' }" @click="$emit('update:active-tab', 'downloading')">
          正在下载
          <span v-if="downloadingCount" class="count">{{ downloadingCount }}</span>
        </div>
        <div class="dl-tab-item" :class="{ active: activeTab === 'completed' }" @click="$emit('update:active-tab', 'completed')">
          传输完成
          <span v-if="completedCount" class="count completed">{{ completedCount }}</span>
        </div>
      </div>
    </div>

    <div class="header-right page-toolbar-right">
      <div class="action-group">
        <template v-if="activeTab === 'downloading'">
          <el-button v-if="downloadingCount" link @click="$emit('cancel-all')"><el-icon><CircleClose /></el-icon>全部取消</el-button>
        </template>
        <template v-else>
          <el-button v-if="completedCount" link @click="$emit('clear-completed')"><el-icon><Delete /></el-icon>清空记录</el-button>
        </template>
        <el-button type="primary" size="default" round @click="$emit('show-add')"><el-icon><Plus /></el-icon>新建下载</el-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { CircleClose, Delete, Plus } from '@element-plus/icons-vue';
defineProps({
  activeTab: { type: String, default: 'downloading' },
  downloadingCount: { type: Number, default: 0 },
  completedCount: { type: Number, default: 0 },
});
defineEmits(['update:active-tab', 'cancel-all', 'clear-completed', 'show-add']);
</script>

<style lang="scss" scoped>
.dl-header { height: 64px; flex-shrink: 0; }
.dl-tabs {
  display: flex; gap: 12px;
  .dl-tab-item {
    font-size: 14px; color: var(--color-text-secondary); cursor: pointer; padding: 8px 14px; font-weight: 500; transition: all 0.3s; display: flex; align-items: center; gap: 6px; border-radius: var(--radius-full);
    &:hover { color: var(--color-primary); background: rgba(var(--color-primary-rgb), 0.05); }
    &.active { color: var(--color-primary); font-weight: 600; background: var(--color-primary-light); }
    .count { font-size: 11px; background: var(--surface-soft); color: var(--color-text-muted); padding: 1px 6px; border-radius: 10px; &.completed { background: var(--success-soft); color: var(--tag-a15-text); } }
  }
}
.action-group {
  display: flex; align-items: center; gap: 12px;
  .el-button--link { color: var(--color-text-secondary); font-size: 14px; .el-icon { margin-right: 4px; font-size: 16px; } &:hover { color: var(--el-color-primary); } }
}
</style>
