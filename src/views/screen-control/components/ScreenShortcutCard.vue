<template>
  <el-card class="custom-card shortcut-card" shadow="hover">
    <template #header>
      <div class="card-header flex-y-center">
        <SmartIcon name="bolt" color="var(--color-warning)" :size="18" :show-background="false" :show-decoration="false" />
        <span>便捷控制</span>
      </div>
    </template>
    <div class="shortcut-btns">
      <el-button v-for="action in actions" :key="action.code" circle class="key-btn" :title="action.title" :disabled="!isStreaming" @click="$emit('keyevent', action.code)">
        <SmartIcon :name="action.icon" :color="action.color" :size="20" :show-decoration="false" />
      </el-button>
    </div>
  </el-card>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  actions: { type: Array, default: () => [] },
  isStreaming: { type: Boolean, default: false },
});

defineEmits(['keyevent']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.shortcut-card {
  margin-bottom: 20px;
}

.shortcut-btns {
  display: flex;
  justify-content: space-around;
  align-items: center;
  padding: 4px 0;
}

.key-btn {
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  padding: 0;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  margin: 0 !important;

  &:hover {
    transform: translateY(-4px);
    box-shadow: none;
  }

  &:active {
    transform: translateY(0);
  }

  :deep(.smart-icon-container) {
    border-radius: 14px;
  }
}
</style>
