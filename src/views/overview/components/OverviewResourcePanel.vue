<template>
  <div class="dashboard-card panel-card resource-panel" v-loading="loading">
    <div class="resource-stage">
      <article
        v-for="metric in metrics"
        :key="metric.label"
        class="resource-side-card"
        :style="getResourceMetricVars(metric)"
      >
        <div class="resource-side-top">
          <div class="resource-side-icon">
            <SmartIcon :name="metric.icon" :color="metric.color" :size="16" />
          </div>
          <span class="resource-side-label">{{ metric.label }}</span>
        </div>

        <strong class="resource-side-value">{{ metric.percent }}%</strong>
        <span class="resource-side-meta">{{ metric.meta }}</span>

        <div class="resource-bar-track">
          <div class="resource-bar-fill" :style="{ width: `${metric.percent}%` }"></div>
        </div>
      </article>

      <article class="resource-side-card resource-battery-card" :style="getResourceMetricVars(batteryMetric)">
        <div class="resource-side-top">
          <div class="resource-side-icon">
            <SmartIcon name="battery_charging" :color="batteryMetric.color" :size="16" />
          </div>
          <span class="resource-side-label">{{ batteryMetric.label }}</span>
        </div>

        <strong class="resource-side-value">{{ batteryMetric.percent }}%</strong>
        <span class="resource-side-meta">{{ batteryStatusText }} · {{ batteryTemp }}°C</span>

        <div class="resource-bar-track">
          <div class="resource-bar-fill" :style="{ width: `${batteryMetric.percent}%` }"></div>
        </div>
      </article>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  metrics: {
    type: Array,
    default: () => [],
  },
  batteryMetric: {
    type: Object,
    required: true,
  },
  batteryStatusText: {
    type: String,
    default: '',
  },
  batteryTemp: {
    type: [Number, String],
    default: 0,
  },
  loading: {
    type: Boolean,
    default: false,
  },
});

const getResourceMetricVars = (metric) => ({
  '--resource-accent': metric.color,
  '--resource-accent-soft': metric.accent,
  '--resource-progress': `${metric.percent}%`,
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.resource-panel {
  width: calc(43%);
  // height: 100%;
  flex-shrink: 0;
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.resource-stage {
  height: 100%;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  align-items: stretch;
}

.resource-side-card {
  @include overview-interactive-panel(
    20px,
    color-mix(in srgb, var(--color-border) 75%, transparent),
    linear-gradient(180deg, color-mix(in srgb, var(--surface-soft) 92%, white 8%), color-mix(in srgb, var(--surface-strong) 90%, transparent)),
    radial-gradient(circle at top, var(--resource-accent-soft), transparent 55%),
    (transform 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease),
    true,
    true
  );
  height: 100%;
  padding: 18px 14px 16px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  min-width: 0;
}

.resource-side-card:hover {
  @include overview-hover-lift;
}

.resource-side-top {
  display: flex;
  align-items: center;
  gap: 10px;
}

.resource-side-icon {
  @include overview-icon-box(34px, 12px, var(--resource-accent-soft), inherit);
}

.resource-side-label {
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 600;
  line-height: 1.3;
}

.resource-side-value {
  font-size: 18px;
  line-height: 1;
  color: var(--color-text-primary);
  font-weight: 800;
}

.resource-side-meta {
  font-size: 11px;
  color: var(--color-text-muted);
  line-height: 1.5;
  word-break: break-all;
}

.resource-bar-track {
  position: relative;
  width: 100%;
  height: 8px;
  border-radius: 999px;
  background: rgba(148, 163, 184, 0.12);
  overflow: hidden;
}

.resource-bar-fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--resource-accent), color-mix(in srgb, var(--resource-accent) 72%, white 28%));
  box-shadow: 0 0 16px color-mix(in srgb, var(--resource-accent) 28%, transparent);
}

.resource-battery-card {
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--surface-soft) 92%, white 8%), color-mix(in srgb, var(--surface-strong) 90%, transparent)),
    radial-gradient(circle at top, color-mix(in srgb, var(--resource-accent-soft) 100%, transparent), transparent 55%);
}

@media (max-width: 768px) {
  .resource-panel {
    width: 100%;
  }

  .resource-stage {
    grid-template-columns: 1fr;
  }
}
</style>
