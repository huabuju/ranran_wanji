<template>
  <section class="dashboard-card summary-panel" v-loading="loading">
    <header class="summary-head">
      <div class="summary-head-title">
        <span class="summary-head-icon" aria-hidden="true">
          <SmartIcon
            name="phone"
            :size="20"
            color="var(--color-primary)"
            :show-background="false"
            :show-decoration="false"
          />
        </span>
        <div class="summary-head-text">
          <span class="summary-kicker">Device Report</span>
          <h2 class="summary-title">设备摘要</h2>
        </div>
      </div>
      <div class="summary-actions">
        <el-dropdown
          trigger="click"
          :disabled="!isConnected"
          @command="$emit('reboot', $event)"
        >
          <button
            class="summary-action-button"
            type="button"
            :disabled="!isConnected"
            :class="{ 'is-disabled': !isConnected }"
          >
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
    </header>

    <div class="summary-body">
      <div class="base-info-grid">
        <div
          v-for="(item, index) in baseItems"
          :key="item.key || item.label"
          class="base-info-row"
          :style="{ '--enter-delay': `${index * 40}ms` }"
        >
          <span class="base-info-label">{{ item.label }}</span>
          <strong class="base-info-value" :title="item.value">{{ item.value }}</strong>
        </div>
      </div>
      <div class="summary-resource-grid">
        <article
          v-for="(metric, index) in resourceMetrics"
          :key="metric.label"
          class="summary-resource-card"
          :style="getResourceMetricVars(metric, baseItems.length + index)"
        >
          <div class="summary-resource-top">
            <span class="summary-resource-icon" aria-hidden="true">
              <SmartIcon
                :name="metric.icon"
                :color="metric.color"
                :size="16"
                :show-background="false"
                :show-decoration="false"
              />
            </span>
            <span class="summary-resource-label">{{ metric.label }}</span>
          </div>
          <div class="summary-resource-value">
            <strong>{{ metric.percent }}</strong>
            <span class="summary-resource-unit">%</span>
          </div>
          <small class="summary-resource-meta">{{ metric.meta }}</small>
          <div
            class="summary-resource-track"
            role="progressbar"
            :aria-valuenow="metric.percent"
            aria-valuemin="0"
            aria-valuemax="100"
          >
            <span
              class="summary-resource-track-fill"
              :style="{ width: `${Math.min(Math.max(Number(metric.percent) || 0, 0), 100)}%` }"
            ></span>
          </div>
        </article>
      </div>
    </div>
  </section>
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

const getResourceMetricVars = (metric, index = 0) => ({
  '--summary-resource-accent': metric.color,
  '--summary-resource-accent-soft': metric.accent,
  '--enter-delay': `${index * 40}ms`,
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
  @include overview-panel-enter;
}

.summary-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
  padding: 18px 18px 16px;
  background:
    radial-gradient(circle at 0% 0%, rgba(var(--color-primary-rgb), 0.14), transparent 38%),
    radial-gradient(circle at 100% 0%, rgba(var(--color-success-rgb), 0.1), transparent 42%),
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
}

.summary-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.summary-head-title {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.summary-head-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(var(--color-primary-rgb), 0.18), rgba(var(--color-primary-rgb), 0.08));
  border: 1px solid rgba(var(--color-primary-rgb), 0.22);
  color: var(--color-primary);
  box-shadow: inset 0 0 0 1px var(--border-strong), 0 6px 14px rgba(var(--color-primary-rgb), 0.18);
  position: relative;

  &::after {
    content: '';
    position: absolute;
    inset: -2px;
    border-radius: inherit;
    box-shadow: 0 0 0 0 rgba(var(--color-primary-rgb), 0.32);
    animation: overview-pulse-ring 2.6s ease-out infinite;
    pointer-events: none;

    @media (prefers-reduced-motion: reduce) {
      animation: none;
    }
  }
}

.summary-head-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.summary-kicker {
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.summary-title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 16px;
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: 0.02em;
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

.summary-action-button:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(var(--color-primary-rgb), 0.22);
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

.wireless-button:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(var(--color-success-rgb), 0.22);
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
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 10px;
  min-width: 0;
  min-height: 150px;
  padding: 14px 14px 14px;
  border: 1px solid color-mix(in srgb, var(--color-border) 75%, transparent);
  border-radius: 18px;
  background:
    radial-gradient(circle at top right, var(--summary-resource-accent-soft), transparent 60%),
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
  overflow: hidden;
  transition: transform 0.22s ease, box-shadow 0.22s ease, border-color 0.22s ease;
  animation: overview-panel-enter 360ms ease-out both;
  animation-delay: var(--enter-delay, 0ms);

  @media (prefers-reduced-motion: reduce) {
    animation: none;
    transition: none;
  }
}

.summary-resource-card::before {
  content: '';
  position: absolute;
  inset: 0 auto auto 0;
  width: 56px;
  height: 56px;
  border-radius: 0 0 100% 0;
  background: linear-gradient(135deg, var(--summary-resource-accent-soft), transparent 70%);
  pointer-events: none;
  opacity: 0.85;
}

.summary-resource-card:hover {
  transform: translateY(-2px);
  border-color: color-mix(in srgb, var(--summary-resource-accent) 50%, var(--color-border));
  box-shadow: 0 14px 28px -16px color-mix(in srgb, var(--summary-resource-accent) 60%, transparent);
}

.summary-resource-top {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 700;
}

.summary-resource-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 9px;
  background: color-mix(in srgb, var(--summary-resource-accent) 18%, transparent);
  color: var(--summary-resource-accent);
}

.summary-resource-label {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.summary-resource-value {
  display: inline-flex;
  align-items: baseline;
  gap: 2px;
  color: var(--color-text-primary);
  line-height: 1;
  font-feature-settings: 'tnum' 1;
}

.summary-resource-value strong {
  font-size: 26px;
  font-weight: 800;
  letter-spacing: -0.02em;
}

.summary-resource-unit {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-secondary);
}

.summary-resource-meta {
  display: block;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.4;
  word-break: break-all;
}

.summary-resource-track {
  position: relative;
  width: 100%;
  height: 8px;
  overflow: hidden;
  border-radius: 999px;
  background: color-mix(in srgb, var(--summary-resource-accent) 14%, rgba(148, 163, 184, 0.18));
}

.summary-resource-track-fill {
  position: relative;
  display: block;
  height: 100%;
  border-radius: inherit;
  background:
    linear-gradient(120deg, transparent 30%, rgba(255, 255, 255, 0.45) 50%, transparent 70%) 0 0 / 220% 100% no-repeat,
    linear-gradient(90deg, var(--summary-resource-accent), color-mix(in srgb, var(--summary-resource-accent) 60%, white 40%));
  background-blend-mode: overlay, normal;
  animation: overview-progress-shimmer 2.4s linear infinite;
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1);

  @media (prefers-reduced-motion: reduce) {
    animation: none;
  }
}

.base-info-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  min-height: 0;
}

.base-info-row {
  position: relative;
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr);
  gap: 8px;
  align-items: center;
  min-height: 36px;
  padding: 8px 12px;
  border-radius: 12px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 5%, transparent), transparent 60%),
    color-mix(in srgb, var(--surface-panel) 86%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-border) 60%, transparent);
  overflow: hidden;
  transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease, background 0.2s ease;
  animation: overview-panel-enter 320ms ease-out both;
  animation-delay: var(--enter-delay, 0ms);

  @media (prefers-reduced-motion: reduce) {
    animation: none;
    transition: none;
  }
}

.base-info-row::before {
  content: '';
  position: absolute;
  left: 0;
  top: 12%;
  bottom: 12%;
  width: 2px;
  border-radius: 0 2px 2px 0;
  background: linear-gradient(180deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 60%, transparent));
  opacity: 0.55;
  transition: opacity 0.2s ease, transform 0.2s ease;
  transform: scaleY(0.6);
  transform-origin: center;
}

.base-info-row:hover {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--color-primary) 32%, var(--color-border));
  box-shadow: 0 10px 22px -16px rgba(var(--color-primary-rgb), 0.32);
}

.base-info-row:hover::before {
  opacity: 1;
  transform: scaleY(1);
}

.base-info-label {
  color: var(--color-text-secondary);
  font-size: 12px;
}

.base-info-value {
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
  .summary-body {
    flex-direction: column;
  }

  .summary-resource-grid {
    width: 100%;
  }

  .base-info-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .base-info-grid {
    grid-template-columns: 1fr;
  }

  .summary-resource-grid {
    grid-template-columns: 1fr;
  }
}
</style>
