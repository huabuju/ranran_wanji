<template>
  <div class="ctrl-card">
    <div class="ctrl-card__header">
      <SmartIcon name="battery_charging" color="var(--color-success)" :size="16" :show-background="false" :show-decoration="false" />
      <span class="ctrl-title">电池模拟</span>
      <el-button type="danger" link size="small" class="header-action" :disabled="!isConnected" @click="$emit('reset-battery')">恢复</el-button>
    </div>
    <div class="ctrl-card__body">
      <div class="field-group">
        <div class="field-label">电量百分比</div>
        <div class="row-inputs">
          <el-input-number v-model="battery.level" class="flex-grow-input" :min="0" :max="100" controls-position="right" :disabled="!isConnected" />
          <el-button type="primary" :disabled="!isConnected" @click="$emit('set-battery-level')">修改</el-button>
        </div>
      </div>

      <div class="field-group">
        <div class="field-label">电池温度 (℃)</div>
        <div class="row-inputs">
          <el-input-number
            v-model="battery.temperature"
            class="flex-grow-input"
            :min="-20"
            :max="80"
            :step="1"
            controls-position="right"
            :disabled="!isConnected"
          />
          <el-button type="primary" :disabled="!isConnected" @click="$emit('set-battery-temperature')">修改</el-button>
        </div>
      </div>

      <div class="divider section-divider" />

      <div class="field-group field-group-tight">
        <div class="field-label">充电模式</div>
        <div class="mode-list">
          <div
            v-for="mode in chargeModes"
            :key="mode.key"
            class="mode-row"
            :class="{ active: battery.mode === mode.key, disabled: !isConnected }"
            @click="isConnected && $emit('set-battery-mode', mode.key)"
          >
            <span class="mode-dot" :style="{ '--mode-dot-color': battery.mode === mode.key ? mode.color : 'var(--color-border)' }"></span>
            <span class="mode-name">{{ mode.label }}</span>
            <div class="mode-toggle" :class="{ on: battery.mode === mode.key }">
              <div class="toggle-knob"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  battery: {
    type: Object,
    required: true,
  },
  chargeModes: {
    type: Array,
    default: () => [],
  },
  isConnected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['reset-battery', 'set-battery-level', 'set-battery-temperature', 'set-battery-mode']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.mode-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.mode-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s, box-shadow 0.2s;
  border: 1px solid transparent;

  &:hover:not(.disabled) {
    background: var(--surface-soft);
  }

  &.active {
    background: rgba(var(--color-primary-rgb), 0.08);
    border-color: rgba(var(--color-primary-rgb), 0.18);
    box-shadow: var(--shadow-elevated-soft);
  }

  &.disabled {
    pointer-events: none;
    opacity: 0.5;
  }

  .mode-dot {
    width: 7px;
    height: 7px;
    background: var(--mode-dot-color);
    border-radius: 50%;
    flex-shrink: 0;
    transition: background 0.2s;
  }

  .mode-name {
    flex: 1;
    font-size: 13px;
    color: var(--color-text-primary);
  }
}

.mode-toggle {
  width: 34px;
  height: 18px;
  border-radius: 9px;
  background: var(--color-border);
  flex-shrink: 0;
  position: relative;
  transition: background 0.25s;

  .toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--surface-strong);
    transition: transform 0.25s;
    box-shadow: var(--shadow-sm);
  }

  &.on {
    background: var(--color-primary);

    .toggle-knob {
      transform: translateX(16px);
    }
  }
}
</style>
