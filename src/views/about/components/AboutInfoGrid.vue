<template>
  <div class="info-grid">
    <div
      v-for="(card, index) in cards"
      :key="index"
      class="info-card"
      :style="getInfoCardVars(card)"
      @click="$emit('card-click', card)"
    >
      <div class="card-icon">
        <SmartIcon :name="card.icon" :size="24" :color="card.color" />
      </div>
      <div class="card-content">
        <span class="label">{{ card.label }}</span>
        <span class="value">{{ card.value }}</span>
        <SmartIcon
          v-if="card.link"
          name="chevron_down"
          :size="14"
          class="external-icon"
          color="var(--color-text-muted)"
          :show-background="false"
          :show-decoration="false"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  cards: { type: Array, default: () => [] },
});

defineEmits(['card-click']);

const getInfoCardVars = (card) => ({
  '--about-card-accent': card.color,
});
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 16px;
  width: 100%;
  max-width: 900px;
  margin-bottom: 48px;
}

.info-card {
  background: var(--bg-glass);
  backdrop-filter: var(--blur-glass);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 24px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  position: relative;
  box-shadow: var(--shadow-card);
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 340ms);
  @include pageCard.overview-main-card-hoverable(var(--bg-glass));

  &:hover .external-icon {
    opacity: 0.7;
    transform: rotate(-45deg) translateY(-1px);
  }
}

.info-card:nth-child(1) {
  --page-enter-delay: 0ms;
}

.info-card:nth-child(2) {
  --page-enter-delay: 40ms;
}

.info-card:nth-child(3) {
  --page-enter-delay: 80ms;
}

.info-card:nth-child(4) {
  --page-enter-delay: 120ms;
}

.card-icon {
  width: 44px;
  height: 44px;
  background: color-mix(in srgb, var(--about-card-accent) 8%, transparent);
  color: var(--about-card-accent);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
}

.label {
  font-size: 12px;
  color: var(--color-text-muted);
}

.value {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.external-icon {
  position: absolute;
  top: 12px;
  right: 12px;
  opacity: 0.3;
  transform: rotate(-45deg);
  transition: opacity 0.2s ease, transform 0.2s ease;
}
</style>
