<template>
  <el-card class="custom-card advanced-settings-card" shadow="hover">
    <template #header>
      <div class="card-header flex-y-center">
        <div class="header-icon">
          <SmartIcon name="monitor" color="var(--color-primary)" :size="16" :show-background="false" :show-decoration="false" />
        </div>
        <div class="header-title">
          <span class="title-main">画面与行为</span>
          <span class="title-sub">{{ activeCount }} / {{ items.length }} 项已开启</span>
        </div>
        <div class="header-counter">
          <span class="counter-value">{{ activeCount }}</span>
        </div>
      </div>
    </template>

    <div class="switch-list">
      <label
        v-for="item in items"
        :key="item.key"
        class="switch-item"
        :class="{ 'is-active': form[item.key], 'is-disabled': isStreaming }"
        :style="{ '--item-accent': item.color, '--item-rgb': item.colorRgb }"
      >
        <span class="item-glow" aria-hidden="true" />

        <span class="item-icon">
          <SmartIcon :name="item.icon" :color="item.color" :size="16" :show-background="false" :show-decoration="false" />
        </span>

        <span class="item-text">
          <span class="item-title">
            {{ item.title }}
            <span class="item-subtitle">{{ item.subtitle }}</span>
          </span>
          <span class="item-desc">{{ item.description }}</span>
        </span>

        <el-switch v-model="form[item.key]" class="panel-switch" :disabled="isStreaming" />
      </label>
    </div>
  </el-card>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  form: { type: Object, required: true },
  items: { type: Array, default: () => [] },
  isStreaming: { type: Boolean, default: false },
});

const activeCount = computed(() => props.items.filter((item) => props.form[item.key]).length);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.advanced-settings-card {
  flex: 1;
  display: flex;
  flex-direction: column;

  :deep(.el-card__body) {
    flex: 1;
  }
}

.header-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 9px;
  background: linear-gradient(135deg, rgba(var(--color-primary-rgb), 0.16), rgba(var(--color-primary-rgb), 0.06));
  border: 1px solid rgba(var(--color-primary-rgb), 0.18);
  margin-right: 10px;
}

.header-title {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
  flex: 1;
  min-width: 0;

  .title-main {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .title-sub {
    font-size: 11px;
    color: var(--color-text-muted);
    letter-spacing: 0.04em;
  }
}

.header-counter {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 32px;
  height: 22px;
  padding: 0 8px;
  border-radius: 999px;
  background: linear-gradient(135deg, rgba(var(--color-primary-rgb), 0.18), rgba(var(--color-primary-rgb), 0.08));
  border: 1px solid rgba(var(--color-primary-rgb), 0.25);
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.switch-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.switch-item {
  position: relative;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 14px;
  padding: 12px 16px;
  background: var(--surface-soft);
  border-radius: 12px;
  border: 1px solid var(--border-soft);
  cursor: pointer;
  overflow: hidden;
  transition:
    background 0.3s ease,
    border-color 0.3s ease,
    transform 0.25s ease,
    box-shadow 0.3s ease;

  &:hover:not(.is-disabled) {
    transform: translateX(2px);
    border-color: rgba(var(--item-rgb), 0.32);
    background: var(--surface-strong);
  }

  &.is-active {
    background: linear-gradient(120deg, rgba(var(--item-rgb), 0.08), rgba(var(--item-rgb), 0.02));
    border-color: rgba(var(--item-rgb), 0.32);
    box-shadow: 0 6px 18px -10px rgba(var(--item-rgb), 0.45);

    .item-icon {
      background: linear-gradient(135deg, rgba(var(--item-rgb), 0.22), rgba(var(--item-rgb), 0.1));
      border-color: rgba(var(--item-rgb), 0.42);
      box-shadow: 0 0 0 4px rgba(var(--item-rgb), 0.08);
    }

    .item-glow {
      opacity: 1;
    }
  }

  &.is-disabled {
    cursor: not-allowed;
    opacity: 0.78;
  }
}

.item-glow {
  position: absolute;
  top: -40%;
  left: -10%;
  width: 60%;
  height: 180%;
  background: radial-gradient(ellipse at center, rgba(var(--item-rgb), 0.18), transparent 65%);
  filter: blur(12px);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.4s ease;
}

.item-icon {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 10px;
  background: var(--surface-chip);
  border: 1px solid var(--border-soft);
  flex-shrink: 0;
  transition: all 0.3s ease;
}

.item-text {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.item-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--color-text-primary);
  display: inline-flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: wrap;
}

.item-subtitle {
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.06em;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.item-desc {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.45;
}

:deep(.panel-switch) {
  --el-switch-on-color: var(--item-accent);
  position: relative;
  z-index: 1;
}
</style>