<template>
  <el-card class="custom-card advanced-settings-card" shadow="hover">
    <template #header>
      <div class="card-header flex-y-center">
        <SmartIcon name="monitor" color="var(--color-primary)" :size="18" :show-background="false" :show-decoration="false" />
        <span>核心画面设置</span>
      </div>
    </template>

    <div class="switch-list">
      <div v-for="item in items" :key="item.key" class="switch-item flex-between">
        <div class="switch-label">
          <span>{{ item.title }}</span>
          <small>{{ item.description }}</small>
        </div>
        <el-switch v-model="form[item.key]" class="panel-switch" :disabled="isStreaming" />
      </div>
    </div>
  </el-card>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  form: { type: Object, required: true },
  items: { type: Array, default: () => [] },
  isStreaming: { type: Boolean, default: false },
});
</script>

<style lang="scss" scoped>
@use './shared.scss';

.advanced-settings-card {
  flex: 1;
  display: flex;
  flex-direction: column;

  :deep(.el-card__body) {
    flex: 1;
  }
}

.switch-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.switch-item {
  padding: 12px 16px;
  background: var(--surface-soft);
  border-radius: 10px;
  border: 1px solid var(--color-divider);
  transition: all 0.2s;

  &:hover {
    background: var(--surface-strong);
    border-color: rgba(var(--color-primary-rgb), 0.2);
  }
}

.switch-label {
  display: flex;
  flex-direction: column;
  gap: 4px;

  span {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  small {
    font-size: 12px;
    color: var(--color-text-muted);
  }
}

:deep(.panel-switch) {
  --el-switch-on-color: var(--color-success);
}
</style>
