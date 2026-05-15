<template>
  <div class="action-section section-fade-in">
    <div class="btn-group">
      <el-button type="primary" class="update-btn" round @click="$emit('check-updates')">
        <span>检测更新</span>
      </el-button>
      <el-button type="info" class="changelog-btn" round :loading="loadingChangelog" @click="$emit('show-changelog')">
        <span>更新日志</span>
      </el-button>
    </div>
  </div>
</template>

<script setup>
defineProps({
  loadingChangelog: { type: Boolean, default: false },
});

defineEmits(['check-updates', 'show-changelog']);
</script>

<style lang="scss" scoped>
.action-section {
  margin-bottom: 48px;
}

.btn-group {
  display: flex;
  gap: 16px;
  justify-content: center;
}

.update-btn,
.changelog-btn {
  height: 44px;
  padding: 0 24px;
  font-weight: 600;
  gap: 8px;
  transition: all 0.3s ease;
}

.update-btn {
  box-shadow: var(--info-shadow-md);

  &:hover {
    transform: translateY(-1px);
    box-shadow: var(--info-shadow-lg);
  }
}

.changelog-btn {
  background: var(--surface-soft);
  border: 1px solid var(--color-border);
  color: var(--color-text-primary);

  &:hover {
    background: var(--surface-chip-hover);
    transform: translateY(-1px);
  }
}
</style>
