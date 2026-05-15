<template>
  <div class="top-action-bar page-toolbar surface-card">
    <div class="breadcrumb">
      <span class="crumb">快捷执行您常用的 ADB / Fastboot 命令</span>
      <div class="drag-tip" :class="{ 'batch-mode': isBatchMode }">
        <el-icon><Rank /></el-icon>
        <span>{{ isBatchMode ? '点击卡片可选择要删除的自定义指令' : '按住卡片并拖拽可调整排序（当前已禁用）' }}</span>
      </div>
    </div>

    <div class="right-actions page-toolbar-right">
      <template v-if="!isBatchMode">
        <el-button class="batch-manage-btn" :disabled="customCommandsCount === 0" @click="$emit('enter-batch-mode')">
          批量管理
        </el-button>
        <el-button type="primary" class="add-btn" @click="$emit('add-command')">
          <el-icon><Plus /></el-icon>
          新增指令
        </el-button>
      </template>

      <template v-else>
        <span class="batch-count">已选 {{ selectedCount }} 项</span>
        <el-button class="batch-manage-btn" :disabled="customCommandsCount === 0" @click="$emit('toggle-select-all')">
          {{ isAllCustomSelected ? '取消全选' : '全选自定义' }}
        </el-button>
        <el-button type="danger" class="batch-delete-btn" :disabled="selectedCount === 0" @click="$emit('batch-delete')">
          批量删除
        </el-button>
        <el-button @click="$emit('exit-batch-mode')">取消</el-button>
      </template>
    </div>
  </div>
</template>

<script setup>
import { Plus, Rank } from '@element-plus/icons-vue';

defineProps({
  isBatchMode: {
    type: Boolean,
    default: false,
  },
  customCommandsCount: {
    type: Number,
    default: 0,
  },
  selectedCount: {
    type: Number,
    default: 0,
  },
  isAllCustomSelected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['enter-batch-mode', 'exit-batch-mode', 'toggle-select-all', 'batch-delete', 'add-command']);
</script>

<style lang="scss" scoped>
.top-action-bar {
  flex-shrink: 0;
  min-height: 52px;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;

  .crumb {
    color: var(--color-text-muted);
  }
}

.drag-tip {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-left: 12px;
  padding: 4px 12px;
  background: rgba(var(--color-primary-rgb), 0.05);
  border-radius: var(--radius-full);
  border: 1px solid rgba(var(--color-primary-rgb), 0.1);
  transition: all 0.3s;
  cursor: help;

  &:hover {
    background: rgba(var(--color-primary-rgb), 0.1);
    color: var(--color-primary);
  }

  &.batch-mode {
    background: rgba(var(--color-danger-rgb), 0.08);
    border-color: rgba(var(--color-danger-rgb), 0.15);
    color: var(--text-danger-strong);
  }

  .el-icon {
    font-size: 14px;
    color: var(--color-primary);
  }
}

.right-actions {
  gap: 10px;

  .batch-count {
    font-size: 13px;
    color: var(--color-text-secondary);
    font-weight: 600;
  }
}

.batch-manage-btn,
.batch-delete-btn,
.add-btn {
  border-radius: var(--radius-full);
  padding: 8px 16px;
  transition: all 0.2s;

  &:hover {
    transform: translateY(-2px);
  }
}

.add-btn {
  box-shadow: 0 4px 12px rgba(var(--color-primary-rgb), 0.3);
}
</style>
