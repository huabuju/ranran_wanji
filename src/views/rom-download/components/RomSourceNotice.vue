<template>
  <div v-if="visible" class="rom-source-notice">
    <SmartIcon name="info" :size="14" color="var(--color-warning)" />
    <span>当前数据来源：{{ sourceLabel }}。{{ sourceDescription }}</span>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  visible: { type: Boolean, default: false },
  sourceLabel: { type: String, default: '' },
  sourceDescription: { type: String, default: '' },
});
</script>

<style lang="scss" scoped>
.rom-source-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: linear-gradient(135deg, rgba(var(--color-warning-rgb), 0.12), rgba(var(--color-warning-rgb), 0.06));
  border: 1px solid rgba(var(--color-warning-rgb), 0.2);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--text-warning);
  line-height: 1.5;
}
</style>
