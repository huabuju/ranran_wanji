<template>
  <div class="dashboard-card summary-panel" v-loading="loading">
    <div class="summary-head">
      <span class="summary-kicker">Device Report</span>
      <div class="summary-actions">
        <el-dropdown
          trigger="click"
          :disabled="!isConnected"
          @command="$emit('reboot', $event)"
        >
          <button class="summary-action-button" type="button" :disabled="!isConnected" :class="{ 'is-disabled': !isConnected }">
            <SmartIcon name="reboot" :size="14" color="currentColor" :show-background="false" :show-decoration="false" />
            高级重启
            <SmartIcon name="chevron_down" :size="12" color="currentColor" :show-background="false" :show-decoration="false" />
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="opt in commandOptions"
                :key="opt.label"
                :command="opt"
              >
                <span class="summary-dropdown-item">
                  <SmartIcon :name="opt.icon" :color="opt.color" :size="14" :show-background="false" :show-decoration="false" />
                  {{ opt.label }}
                </span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <button class="wireless-button" type="button" @click="$emit('wireless-adb')">
          <SmartIcon name="wifi" :size="14" color="currentColor" :show-background="false" :show-decoration="false" />
          无线调试
        </button>
      </div>
    </div>

    <div class="summary-body">
      <div class="base-info-grid">
        <div
          v-for="item in baseItems"
          :key="item.key || item.label"
          class="base-info-row"
        >
          <span>{{ item.label }}</span>
          <strong :title="item.value">{{ item.value }}</strong>
        </div>
      </div>
      <div class="summary-resource-grid">
        <article
          v-for="metric in resourceMetrics"
          :key="metric.label"
          class="summary-resource-card"
          :style="getResourceMetricVars(metric)"
        >
          <div class="summary-resource-top">
            <SmartIcon :name="metric.icon" :color="metric.color" :size="16" />
            <span>{{ metric.label }}</span>
          </div>
          <strong>{{ metric.percent }}%</strong>
          <small>{{ metric.meta }}</small>
          <div class="summary-resource-track">
            <span :style="{ width: `${metric.percent}%` }"></span>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  loading: {
    type: Boolean,
    default: false,
  },
  resourceMetrics: {
    type: Array,
    default: () => [],
  },
  baseItems: {
    type: Array,
    default: () => [],
  },
  commandOptions: {
    type: Array,
    default: () => [],
  },
  isConnected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['reboot', 'wireless-adb']);

const getResourceMetricVars = (metric) => ({
  '--summary-resource-accent': metric.color,
  '--summary-resource-accent-soft': metric.accent,
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.summary-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
  padding: 16px;
  background:
    radial-gradient(circle at top left, rgba(var(--color-primary-rgb), 0.1), transparent 34%),
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
}

.summary-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.summary-kicker {
  display: inline-flex;
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.summary-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

.summary-action-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border: 1px solid rgba(var(--color-primary-rgb), 0.24);
  border-radius: 10px;
  background: linear-gradient(180deg, var(--color-primary-light), color-mix(in srgb, var(--color-primary-light) 62%, var(--surface-elevated-strong)));
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}

.summary-action-button:hover:not(.is-disabled) {
  transform: translateY(-1px);
  border-color: rgba(var(--color-primary-rgb), 0.36);
  box-shadow: 0 10px 20px rgba(var(--color-primary-rgb), 0.14);
}

.summary-action-button.is-disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.summary-dropdown-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 120px;
}

.wireless-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border: 1px solid rgba(var(--color-success-rgb), 0.24);
  border-radius: 10px;
  background: linear-gradient(180deg, var(--success-soft), color-mix(in srgb, var(--success-soft) 62%, var(--surface-elevated-strong)));
  color: var(--color-success);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.18s ease;
}

.wireless-button:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--color-success-rgb), 0.36);
  box-shadow: 0 10px 20px rgba(var(--color-success-rgb), 0.14);
}

.summary-body {
  display: flex;
  width: 100%;
  gap: 14px;
  min-height: 0;
}

.summary-resource-grid {
  width: 46%;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.summary-resource-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  min-width: 0;
  min-height: 150px;
  padding: 16px 12px 14px;
  border: 1px solid color-mix(in srgb, var(--color-border) 75%, transparent);
  border-radius: 18px;
  background:
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated)),
    radial-gradient(circle at top, var(--summary-resource-accent-soft), transparent 55%);
}

.summary-resource-top {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.summary-resource-card strong {
  color: var(--color-text-primary);
  font-size: 16px;
  line-height: 1;
}

.summary-resource-card small {
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.4;
  text-align: center;
  word-break: break-all;
}

.summary-resource-track {
  width: 100%;
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: rgba(148, 163, 184, 0.12);
}

.summary-resource-track span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--summary-resource-accent), color-mix(in srgb, var(--summary-resource-accent) 72%, white 28%));
}

.base-info-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  min-height: 0;
}

.base-info-row {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr);
  gap: 8px;
  align-items: center;
  min-height: 34px;
  padding: 8px 10px;
  border-radius: 12px;
  background: color-mix(in srgb, var(--surface-panel) 86%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-border) 60%, transparent);
}

.base-info-row span {
  color: var(--color-text-secondary);
  font-size: 12px;
}

.base-info-row strong {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 700;
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 1080px) {
  .base-info-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .summary-resource-grid {
    grid-template-columns: 1fr;
  }
}
</style>
