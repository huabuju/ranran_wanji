<template>
  <section class="dashboard-card inspection-report" v-loading="loading">
    <header class="inspection-main-head">
      <div class="inspection-head-title">
        <span class="inspection-head-icon" aria-hidden="true">
          <SmartIcon
            name="info"
            :size="18"
            color="var(--brand-violet)"
            :show-background="false"
            :show-decoration="false"
          />
        </span>
        <div class="inspection-head-text">
          <span class="inspection-kicker">Inspection Items</span>
          <h2 class="inspection-title">检测项明细</h2>
        </div>
      </div>
      <span class="inspection-count">
        <span>{{ items.length }}</span>
        <small>项</small>
      </span>
    </header>

    <div class="report-table">
      <div
        v-for="(column, columnIndex) in reportColumns"
        :key="columnIndex"
        class="report-column"
      >
        <div
          v-for="(item, rowIndex) in column"
          :key="item.key || item.label"
          class="report-info-item"
          :style="{ '--enter-delay': `${(columnIndex * column.length + rowIndex) * 40}ms` }"
        >
          <span class="report-item-name">{{ item.label }}</span>
          <strong class="report-value" :title="item.currentValue">{{ item.currentValue }}</strong>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  loading: {
    type: Boolean,
    default: false,
  },
  items: {
    type: Array,
    default: () => [],
  },
});

const reportColumns = computed(() => {
  const total = props.items.length;
  if (!total) return [[], [], [], []];
  const columnCount = 4;
  const baseSize = Math.ceil(total / columnCount);
  const result = [];
  for (let i = 0; i < columnCount; i += 1) {
    result.push(props.items.slice(i * baseSize, (i + 1) * baseSize));
  }
  return result;
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
  @include overview-panel-enter(60ms);
}

.inspection-report {
  display: flex;
  flex-direction: column;
  gap: 14px;
  width: 100%;
  min-height: 0;
  flex-shrink: 0;
  padding: 18px 18px 16px;
  background:
    radial-gradient(circle at 100% 0%, rgba(168, 85, 247, 0.1), transparent 36%),
    radial-gradient(circle at 0% 0%, rgba(var(--color-primary-rgb), 0.08), transparent 32%),
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
}

.inspection-main-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.inspection-head-title {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.inspection-head-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 11px;
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.18), rgba(168, 85, 247, 0.06));
  border: 1px solid rgba(168, 85, 247, 0.24);
  color: var(--brand-violet);
  box-shadow: inset 0 0 0 1px var(--border-strong);
}

.inspection-head-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.inspection-kicker {
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.inspection-title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 15px;
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: 0.02em;
}

.inspection-count {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
  color: var(--color-primary);
  font-size: 13px;
  font-weight: 800;
  font-feature-settings: 'tnum' 1;
}

.inspection-count small {
  font-size: 10px;
  font-weight: 700;
  color: var(--color-text-secondary);
  letter-spacing: 0.08em;
}

.report-table {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  min-height: 0;
  flex-shrink: 0;
  overflow: visible;
}

.report-column {
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--color-border) 70%, transparent);
  border-radius: 14px;
  background:
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
  transition: border-color 0.22s ease, box-shadow 0.22s ease;
}

.report-column:hover {
  border-color: color-mix(in srgb, var(--color-primary) 26%, var(--color-border));
  box-shadow: 0 12px 28px -22px rgba(var(--color-primary-rgb), 0.36);
}

.report-info-item {
  position: relative;
  display: grid;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  min-height: 38px;
  padding: 0 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--color-border) 38%, transparent);
  transition: background 0.18s ease, transform 0.18s ease;
  animation: overview-panel-enter 280ms ease-out both;
  animation-delay: var(--enter-delay, 0ms);

  @media (prefers-reduced-motion: reduce) {
    animation: none;
    transition: none;
  }
}

.report-info-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 22%;
  bottom: 22%;
  width: 2px;
  border-radius: 0 2px 2px 0;
  background: linear-gradient(180deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 50%, transparent));
  opacity: 0;
  transform: scaleY(0.6);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.report-info-item:nth-child(odd) {
  background: color-mix(in srgb, var(--surface-panel) 64%, transparent);
}

.report-info-item:hover {
  background: color-mix(in srgb, var(--color-primary) 6%, var(--surface-panel));
}

.report-info-item:hover::before {
  opacity: 1;
  transform: scaleY(1);
}

.report-info-item:last-child {
  border-bottom: 0;
}

.report-item-name {
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 700;
}

.report-value {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 700;
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-feature-settings: 'tnum' 1;
}

@media (max-width: 768px) {
  .inspection-report {
    padding: 14px;
  }

  .report-table {
    grid-template-columns: 1fr;
  }
}
</style>
