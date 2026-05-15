<template>
  <div class="ctrl-card">
    <div class="ctrl-card__header">
      <SmartIcon name="notification" color="var(--brand-primary-strong)" :size="16" :show-background="false" :show-decoration="false" />
      <span class="ctrl-title">状态栏图标隐藏</span>
      <span class="header-hint">通过 icon_blacklist 实现，部分 ROM 可能不支持</span>
      <div class="icon-header-right">
        <span v-if="hiddenCount > 0" class="hidden-badge">已隐藏 {{ hiddenCount }} 个</span>
        <el-button size="small" class="header-action" :disabled="!isConnected" @click="$emit('toggle-select-all')">
          {{ allSelected ? '全不选' : '全选' }}
        </el-button>
      </div>
    </div>
    <div class="ctrl-card__body">
      <div class="icon-check-grid">
        <el-checkbox
          v-for="icon in statusIcons"
          :key="icon.key"
          v-model="icon.hidden"
          class="icon-check-item"
          :class="{ checked: icon.hidden, disabled: !isConnected }"
          :disabled="!isConnected"
        >
          <span class="check-label">{{ icon.label }}</span>
        </el-checkbox>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  statusIcons: {
    type: Array,
    default: () => [],
  },
  hiddenCount: {
    type: Number,
    default: 0,
  },
  allSelected: {
    type: Boolean,
    default: false,
  },
  isConnected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['toggle-select-all']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.icon-header-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 10px;

  .hidden-badge {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.08);
    border: 1px solid rgba(var(--color-danger-rgb), 0.2);
    border-radius: 20px;
    padding: 2px 10px;
  }
}

.icon-check-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 4px;
}

.icon-check-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s;
  user-select: none;
  margin-right: 0;

  &:hover:not(.disabled) {
    background: var(--surface-soft);
  }

  &.checked {
    background: rgba(var(--color-danger-rgb), 0.06);

    .check-label {
      color: var(--color-danger);
      font-weight: 500;
    }
  }

  &.disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  :deep(.el-checkbox__input) {
    flex-shrink: 0;
  }

  :deep(.el-checkbox__inner) {
    width: 16px;
    height: 16px;
    border-radius: 4px;
    border-color: var(--color-border);
    background: transparent;
  }

  :deep(.el-checkbox__input.is-checked .el-checkbox__inner) {
    background: var(--color-danger);
    border-color: var(--color-danger);
  }

  :deep(.el-checkbox__label) {
    padding-left: 0;
  }

  .check-label {
    font-size: 12.5px;
    color: var(--color-text-secondary);
    line-height: 1.2;
    transition: color 0.2s;
  }
}
</style>
