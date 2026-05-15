<template>
  <div class="dashboard-card panel-card extra-info-panel" v-loading="loading">
    <div class="extra-list">
      <div
        v-for="panel in cards"
        :key="panel.key"
        class="picture-style-row"
      >
        <div class="p-left">
          <div class="p-icon-box">
            <SmartIcon :name="panel.icon" :size="20" color="currentColor" />
          </div>
          <div class="p-titles">
            <span class="p-title">{{ panel.title }}</span>
            <span class="p-subtitle">{{ panel.value }}</span>
          </div>
        </div>
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
  loading: {
    type: Boolean,
    default: false,
  },
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.extra-info-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  padding: 20px 24px;
}

.extra-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.picture-style-row {
  width: 100%;
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px 16px;
  padding: 16px 20px;
  @include overview-interactive-panel(
    12px,
    var(--color-border),
    var(--surface-soft),
    null,
    (transform 0.2s, box-shadow 0.2s, border-color 0.2s)
  );
}

.picture-style-row:hover {
  @include overview-hover-lift;
}

.p-left {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1 1 360px;
  min-width: 0;
}

.p-icon-box {
  @include overview-icon-box;
}

.p-titles {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.p-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.p-subtitle {
  font-size: 12px;
  color: var(--color-text-muted);
  max-width: 100%;
  white-space: normal;
  overflow-wrap: anywhere;
  word-break: break-word;
  line-height: 1.6;
}

@media (max-width: 768px) {
  .picture-style-row {
    padding: 14px 16px;
  }

  .p-left {
    flex-basis: 100%;
    gap: 12px;
  }
}
</style>
