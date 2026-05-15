<template>
  <div class="top-cards-grid">
    <div
      v-for="(card, index) in cards"
      :key="index"
      class="dashboard-card stat-card"
      v-loading="loadingInfo && index < 6 || loadingApp && index >= 6"
      :style="getStatCardVars(card)"
    >
      <div class="card-top">
        <div class="icon-box" :style="getCardVars(card)">
          <SmartIcon :name="card.icon" :color="card.color" :size="20" />
        </div>
        <span class="card-label">{{ card.label }}</span>
      </div>
      <div class="card-middle">
        <span class="card-value" :class="{ 'is-empty': card.value === '--' }">{{ card.value }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  cards: {
    type: Array,
    default: () => [],
  },
  loadingInfo: {
    type: Boolean,
    default: false,
  },
  loadingApp: {
    type: Boolean,
    default: false,
  },
});

const getCardVars = (card) => ({
  '--overview-card-accent-bg': card.bgColor,
});

const getStatCardVars = (card) => ({
  '--stat-accent-color': card.color || 'var(--color-primary)',
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.dashboard-card.stat-card:hover {
  @include overview-hover-lift;
}

.top-cards-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  flex-shrink: 0;
  padding: 20px 0;
  box-sizing: border-box;
}

.stat-card {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  @include overview-soft-panel(
    20px,
    color-mix(in srgb, var(--color-border) 75%, transparent),
    radial-gradient(circle at top left, color-mix(in srgb, var(--stat-accent-color) 20%, transparent), transparent 70%),
    linear-gradient(180deg, color-mix(in srgb, var(--surface-soft) 92%, white 8%), color-mix(in srgb, var(--surface-strong) 90%, transparent))
  );
}

.card-top {
  display: flex;
  align-items: center;
  gap: 10px;
}

.icon-box {
  @include overview-icon-box(32px, 8px, var(--overview-card-accent-bg), inherit);
}

.card-label {
  font-size: 13px;
  color: var(--color-text-muted);
  font-weight: 500;
}

.card-value {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary);
  line-height: 1.3;
  font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
  word-break: break-all;
}

.card-value.is-empty {
  color: var(--color-text-muted);
}

@media (max-width: 768px) {
  .top-cards-grid {
    grid-template-columns: 1fr;
  }
}
</style>
