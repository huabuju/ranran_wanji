<template>
  <div v-if="visible" class="bottom-notice">
    <SmartIcon name="info" :size="14" color="var(--color-warning)" />
    <span>提示：getprop 属性通常为只读，部分 ROM 需要 Root 权限才能通过 setprop 修改。</span>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  visible: { type: Boolean, default: false },
});
</script>

<style lang="scss" scoped>
.bottom-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: var(--warning-soft);
  border: 1px solid rgba(var(--color-warning-rgb), 0.24);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--text-warning);
  line-height: 1.5;
}
</style>
