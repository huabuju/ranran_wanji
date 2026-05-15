<template>
  <el-card class="custom-card header-card" shadow="hover">
    <div class="header-content">
      <div class="header-actions">
        <el-button type="danger" class="action-btn stop-btn" round :disabled="!isStreaming || !isConnected" @click="$emit('stop')">
          <span class="btn-text">停止投屏</span>
        </el-button>
        <el-button type="primary" class="action-btn launch-btn" round :loading="isLaunching" :disabled="isStreaming || !isConnected" @click="$emit('launch')">
          <span class="btn-text">开始投屏</span>
        </el-button>
      </div>
    </div>
  </el-card>
</template>

<script setup>
defineProps({
  isLaunching: { type: Boolean, default: false },
  isStreaming: { type: Boolean, default: false },
  isConnected: { type: Boolean, default: false },
});

defineEmits(['launch', 'stop']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.header-card {
  :deep(.el-card__body) {
    padding: 20px;
  }
}

.header-content {
  display: flex;
  justify-content: center;
}

.header-actions {
  display: flex;
  gap: 16px;
  width: 100%;
}

.action-btn {
  flex: 1;
  padding: 10px 16px !important;
  font-size: 14px;
  font-weight: 600;
  border: none !important;
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275) !important;

  &:hover {
    transform: translateY(-2px);
  }

  &:active {
    transform: translateY(0);
  }

  &.is-disabled {
    transform: none !important;
    opacity: 0.7;
  }
}

.launch-btn {
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-hover)) !important;
  box-shadow: var(--shadow-primary) !important;

  &:hover {
    box-shadow: 0 12px 24px rgba(var(--color-primary-rgb), 0.28) !important;
  }
}

.stop-btn {
  background: linear-gradient(135deg, var(--color-danger), var(--text-danger-strong)) !important;
  box-shadow: 0 4px 12px rgba(var(--color-danger-rgb), 0.22) !important;
  color: var(--text-on-primary) !important;

  &:hover {
    box-shadow: 0 12px 24px rgba(var(--color-danger-rgb), 0.32) !important;
  }
}
</style>
