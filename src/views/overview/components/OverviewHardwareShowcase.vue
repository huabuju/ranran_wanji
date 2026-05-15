<template>
  <div class="dashboard-card panel-card hardware-showcase" v-loading="loading">
    <div class="hardware-hero">
      <div class="hardware-hero-copy">
        <button class="wireless-debug-tag" type="button" @click="$emit('wireless-adb')">
          无线调试
        </button>

        <span class="hardware-kicker">Device Profile</span>
        <h3 class="hardware-title">{{ title }}</h3>
        <p class="hardware-description">{{ description }}</p>

        <div class="hardware-highlight-list">
          <div
            v-for="item in highlights"
            :key="item.label"
            class="hardware-highlight-pill"
          >
            <span class="hardware-highlight-label">{{ item.label }}</span>
            <strong class="hardware-highlight-value">{{ item.value }}</strong>
          </div>
        </div>
      </div>

      <div class="hardware-visual">
        <div class="hardware-orbit hardware-orbit-a"></div>
        <div class="hardware-orbit hardware-orbit-b"></div>

        <div class="hardware-phone-frame">
          <div class="hardware-phone-screen">
            <div class="hardware-screen-top">
              <span class="hardware-screen-dot"></span>
              <span class="hardware-screen-chip">LIVE DEVICE</span>
            </div>

            <div class="hardware-screen-core">
              <div class="hardware-core-icon">
                <SmartIcon name="device" :size="34" color="currentColor" />
              </div>
              <strong class="hardware-core-brand">ANDROID</strong>
              <span class="hardware-core-meta">Hardware Lens</span>
            </div>

            <div class="hardware-screen-footer">
              <span>READY</span>
              <span>MONITOR</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="hardware-specs-grid">
      <div class="hardware-spec-column" v-for="(group, columnIndex) in columns" :key="columnIndex">
        <div class="hw-item-card" v-for="item in group" :key="item.label">
          <span class="hw-label">{{ item.label }}</span>
          <div class="hw-value-wrap">
            <span class="hw-value">{{ item.value }}</span>
          </div>
        </div>
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
  title: {
    type: String,
    default: '',
  },
  description: {
    type: String,
    default: '',
  },
  highlights: {
    type: Array,
    default: () => [],
  },
  columns: {
    type: Array,
    default: () => [],
  },
});

defineEmits(['wireless-adb']);
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.hardware-showcase {
  --hardware-panel-border: color-mix(in srgb, var(--color-border) 88%, rgba(255, 255, 255, 0.08));
  --hardware-panel-surface:
    linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface-elevated-strong) 88%, rgba(255, 255, 255, 0.08)),
      color-mix(in srgb, var(--surface-panel) 96%, transparent)
    );
  --hardware-panel-accent: radial-gradient(circle at left top, rgba(var(--color-primary-rgb), 0.12), transparent 54%);
  --hardware-panel-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    0 12px 28px rgba(15, 23, 42, 0.08);
  --hardware-panel-hover-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 18px 38px rgba(15, 23, 42, 0.14);
  padding: 15px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  flex-shrink: 0;
  min-height: 520px;
  isolation: isolate;
  background:
    radial-gradient(circle at top left, rgba(var(--color-primary-rgb), 0.12), transparent 34%),
    radial-gradient(circle at 88% 12%, rgba(var(--color-info-rgb), 0.12), transparent 30%),
    linear-gradient(180deg, color-mix(in srgb, var(--surface-soft) 94%, white 6%), color-mix(in srgb, var(--surface-strong) 92%, transparent));
}

.hardware-hero {
  display: grid;
  grid-template-columns: minmax(0, 1.32fr) minmax(220px, 0.68fr);
  gap: 18px;
  align-items: start;
}

.hardware-hero-copy {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
  padding: 6px 2px 0;
}

.wireless-debug-tag {
  position: absolute;
  top: -2px;
  right: 6px;
  min-width: 92px;
  height: 42px;
  padding: 0 18px;
  border: none;
  border-radius: 12px;
  background: linear-gradient(180deg, #e8fbf3 0%, #dbf8ee 100%);
  color: #13b88e;
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.01em;
  cursor: pointer;
  box-shadow:
    0 10px 24px rgba(19, 184, 142, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.78);
  transition: transform 0.18s ease, box-shadow 0.18s ease, filter 0.18s ease;

  &:hover {
    transform: translateY(-1px);
    box-shadow:
      0 14px 28px rgba(19, 184, 142, 0.18),
      inset 0 1px 0 rgba(255, 255, 255, 0.82);
    filter: saturate(1.02);
  }
}

.hardware-kicker {
  display: inline-flex;
  align-self: flex-start;
  padding: 6px 12px;
  border-radius: 999px;
  background: rgba(var(--color-primary-rgb), 0.08);
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.hardware-title {
  margin: 12px 0 8px;
  font-size: 32px;
  line-height: 1.04;
  font-weight: 800;
  letter-spacing: -0.03em;
  color: var(--color-text-primary);
}

.hardware-description {
  margin: 0;
  max-width: 560px;
  font-size: 13px;
  line-height: 1.8;
  color: var(--color-text-secondary);
}

.hardware-highlight-list {
  display: flex;
  flex-wrap: nowrap;
  gap: 12px;
  margin-top: 20px;
}

.hardware-highlight-pill {
  flex: 1;
  min-width: 0;
  padding: 13px 15px;
  @include overview-interactive-panel(
    18px,
    var(--hardware-panel-border),
    var(--hardware-panel-surface),
    var(--hardware-panel-accent),
    (box-shadow 0.22s ease, border-color 0.22s ease, transform 0.22s ease)
  );
  box-shadow: var(--hardware-panel-shadow);
  backdrop-filter: blur(16px);
}

.hardware-highlight-label {
  display: block;
  margin-bottom: 6px;
  font-size: 11px;
  color: var(--color-text-muted);
}

.hardware-highlight-value {
  display: block;
  font-size: 15px;
  line-height: 1.45;
  color: var(--color-text-primary);
  width: 100%;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.hardware-visual {
  position: relative;
  min-height: 232px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 24px;
  overflow: hidden;
  background:
    radial-gradient(circle at 50% 42%, rgba(255, 255, 255, 0.86), rgba(255, 255, 255, 0.34) 36%, transparent 72%),
    linear-gradient(135deg, rgba(var(--color-primary-rgb), 0.14), rgba(var(--color-info-rgb), 0.1) 48%, rgba(var(--color-success-rgb), 0.08));
  border: 1px solid color-mix(in srgb, var(--color-border) 82%, white 10%);
}

.hardware-orbit {
  position: absolute;
  border-radius: 999px;
  filter: blur(2px);
}

.hardware-orbit-a {
  width: 180px;
  height: 180px;
  background: radial-gradient(circle, rgba(var(--color-primary-rgb), 0.22), transparent 68%);
  top: 10px;
  right: -20px;
}

.hardware-orbit-b {
  width: 140px;
  height: 140px;
  background: radial-gradient(circle, rgba(var(--color-info-rgb), 0.18), transparent 68%);
  left: -10px;
  bottom: 8px;
}

.hardware-phone-frame {
  position: relative;
  width: 176px;
  height: 208px;
  padding: 10px;
  border-radius: 28px;
  background: linear-gradient(180deg, rgba(21, 30, 54, 0.96), rgba(43, 54, 86, 0.92));
  box-shadow:
    0 20px 44px rgba(54, 85, 190, 0.2),
    inset 0 0 0 1px rgba(255, 255, 255, 0.12),
    inset 0 -12px 20px rgba(255, 255, 255, 0.04);
  transform: rotate(-7deg) translateX(8px);
}

.hardware-phone-screen {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 20px;
  padding: 14px 14px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background:
    radial-gradient(circle at top, rgba(255, 255, 255, 0.18), transparent 42%),
    linear-gradient(180deg, rgba(15, 23, 42, 0.9), rgba(36, 48, 86, 0.96));
  color: #f8fbff;
  overflow: hidden;
}

.hardware-screen-top,
.hardware-screen-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  font-size: 10px;
  color: rgba(244, 247, 255, 0.72);
}

.hardware-screen-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: linear-gradient(135deg, #84a8ff, #7bf2de);
  box-shadow: 0 0 12px rgba(123, 242, 222, 0.8);
}

.hardware-screen-chip {
  max-width: 84px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hardware-screen-core {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 8px;
}

.hardware-core-icon {
  @include overview-icon-box-elevated(
    56px,
    18px,
    linear-gradient(135deg, rgba(122, 156, 255, 0.3), rgba(115, 233, 209, 0.16)),
    #9bb4ff,
    (
      inset 0 0 0 1px rgba(255, 255, 255, 0.12),
      0 18px 36px rgba(47, 73, 153, 0.35)
    )
  );
}

.hardware-core-brand {
  font-size: 18px;
  line-height: 1.1;
  font-weight: 800;
  letter-spacing: -0.03em;
}

.hardware-core-meta {
  max-width: 116px;
  font-size: 11px;
  line-height: 1.45;
  color: rgba(244, 247, 255, 0.7);
  word-break: break-word;
}

.hardware-specs-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.hardware-spec-column {
  display: grid;
  gap: 12px;
}

.hw-item-card {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  min-height: 66px;
  padding: 14px 16px;
  @include overview-interactive-panel(
    18px,
    var(--hardware-panel-border),
    var(--hardware-panel-surface),
    var(--hardware-panel-accent),
    (transform 0.22s ease, border-color 0.22s ease, box-shadow 0.22s ease)
  );
  box-shadow: var(--hardware-panel-shadow);
  backdrop-filter: blur(16px);
}

.hw-item-card:hover {
  @include overview-hover-lift(
    -2px,
    color-mix(in srgb, var(--color-primary) 44%, var(--hardware-panel-border)),
    var(--hardware-panel-hover-shadow)
  );
}

.hardware-highlight-pill:hover {
  @include overview-hover-lift(
    -2px,
    color-mix(in srgb, var(--color-primary) 44%, var(--hardware-panel-border)),
    var(--hardware-panel-hover-shadow)
  );
}

.hw-label {
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-text-muted);
  font-weight: 600;
  flex-shrink: 0;
  width: 88px;
  padding-top: 2px;
}

.hw-value-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.hw-value {
  font-size: 14px;
  line-height: 1.55;
  color: var(--color-text-primary);
  font-weight: 700;
  text-align: right;
  word-break: break-all;
}

@media (max-width: 768px) {
  .hardware-showcase {
    padding: 18px;
    min-height: auto;
  }

  .hardware-hero,
  .hardware-specs-grid {
    grid-template-columns: 1fr;
  }

  .hardware-highlight-list {
    flex-wrap: wrap;
  }

  .hardware-title {
    font-size: 24px;
  }

  .wireless-debug-tag {
    position: static;
    align-self: flex-start;
    margin-bottom: 12px;
  }

  .hardware-visual {
    min-height: 230px;
  }

  .hardware-phone-frame {
    width: 190px;
    height: 220px;
    transform: rotate(-4deg);
  }

  .hw-item-card {
    align-items: center;
  }

  .hw-label {
    width: auto;
  }
}

:global(html.dark) .hardware-showcase {
  --hardware-panel-border: color-mix(in srgb, var(--color-border) 94%, rgba(165, 180, 252, 0.16));
  --hardware-panel-surface:
    linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.08),
      rgba(15, 23, 42, 0.7)
    );
  --hardware-panel-accent:
    radial-gradient(circle at left top, rgba(var(--color-primary-rgb), 0.16), transparent 52%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.04), transparent 55%);
  --hardware-panel-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    0 14px 32px rgba(2, 6, 23, 0.26);
  --hardware-panel-hover-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.12),
    0 20px 40px rgba(2, 6, 23, 0.34);
}
</style>
