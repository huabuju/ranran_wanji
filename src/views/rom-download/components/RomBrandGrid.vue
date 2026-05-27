<template>
  <div class="brand-grid">
    <div v-for="brand in brands" :key="brand.key" class="brand-card" @click="$emit('select-brand', brand)">
      <div class="brand-icon">
        <SmartIcon name="device" :color="brand.color || 'var(--color-primary)'" :size="22" :show-background="false" :show-decoration="false" />
      </div>
      <div class="brand-info">
        <div class="brand-name">{{ brand.name }}</div>
        <div class="brand-meta">{{ brand.models.length }} 款机型</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';
defineProps({ brands: { type: Array, default: () => [] } });
defineEmits(['select-brand']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.brand-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; }
.brand-card {
  background: var(--bg-glass);
  backdrop-filter: var(--blur-glass);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 18px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-shadow: var(--shadow-card);
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 340ms);
  @include pageCard.overview-main-card-hoverable(var(--bg-glass));
}

.brand-card:nth-child(1) {
  --page-enter-delay: 0ms;
}

.brand-card:nth-child(2) {
  --page-enter-delay: 40ms;
}

.brand-card:nth-child(3) {
  --page-enter-delay: 80ms;
}

.brand-card:nth-child(4) {
  --page-enter-delay: 120ms;
}

.brand-card:nth-child(5) {
  --page-enter-delay: 160ms;
}

.brand-card:nth-child(6) {
  --page-enter-delay: 200ms;
}
.brand-icon {
  width: 28px; height: 28px; background: var(--color-primary-light); border-radius: 8px;
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
}
.brand-name { font-size: 14px; font-weight: 700; color: var(--color-text-primary); }
.brand-meta { font-size: 11px; color: var(--color-text-muted); margin-top: 2px; }
</style>
