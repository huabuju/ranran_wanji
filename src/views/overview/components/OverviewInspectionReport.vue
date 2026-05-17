<template>
  <section class="dashboard-card inspection-report" v-loading="loading">
    <div class="inspection-main-head">
      <span class="inspection-kicker">Inspection Items</span>
    </div>

    <div class="report-table">
      <div
        v-for="(column, columnIndex) in reportColumns"
        :key="columnIndex"
        class="report-column"
      >
        <div
          v-for="item in column"
          :key="item.key || item.label"
          class="report-info-item"
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
  const middleIndex = Math.ceil(props.items.length / 2);
  return [
    props.items.slice(0, middleIndex),
    props.items.slice(middleIndex),
  ];
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.inspection-report {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  min-height: 0;
  flex-shrink: 0;
  padding: 16px;
  background:
    radial-gradient(circle at top left, rgba(var(--color-primary-rgb), 0.08), transparent 32%),
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
}

.inspection-main-head {
  display: flex;
  align-items: flex-start;
}

.inspection-kicker {
  display: inline-flex;
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.report-table {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
  min-height: 0;
  flex-shrink: 0;
  overflow: visible;
}

.report-column {
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--color-border) 70%, transparent);
  border-radius: 14px;
  background: var(--surface-elevated);
}

.report-info-item {
  display: grid;
  grid-template-columns: 104px minmax(0, 1fr);
  gap: 12px;
  align-items: center;
  min-height: 38px;
  padding: 0 14px;
  border-bottom: 1px solid color-mix(in srgb, var(--color-border) 38%, transparent);
}

.report-info-item:nth-child(odd) {
  background: color-mix(in srgb, var(--surface-panel) 64%, transparent);
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
}

@media (max-width: 768px) {
  .inspection-report {
    padding: 12px;
  }

  .report-table {
    grid-template-columns: 1fr;
  }
}
</style>
