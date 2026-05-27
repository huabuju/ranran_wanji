<template>
  <div class="tech-tags section-fade-in">
    <span v-for="tag in tags" :key="tag" class="tag">{{ tag }}</span>
  </div>
</template>

<script setup>
defineProps({
  tags: { type: Array, default: () => [] },
});
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.tech-tags {
  display: flex;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 40px;
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 360ms);
}

.tag {
  padding: 4px 12px;
  background: var(--surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 20px;
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 500;
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 300ms);
}

.tag:nth-child(1) {
  --page-enter-delay: 0ms;
}

.tag:nth-child(2) {
  --page-enter-delay: 40ms;
}

.tag:nth-child(3) {
  --page-enter-delay: 80ms;
}

.tag:nth-child(4) {
  --page-enter-delay: 120ms;
}
</style>
