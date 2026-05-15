<template>
  <div class="ctrl-card">
    <div class="ctrl-card__header">
      <SmartIcon name="lock_screen" color="var(--status-recovery)" :size="16" :show-background="false" :show-decoration="false" />
      <span class="ctrl-title">锁屏时间</span>
    </div>
    <div class="ctrl-card__body">
      <div v-if="lockscreen.current > 0 || lockscreen.current === -1" class="info-strip">
        <span>当前：<b>{{ lockscreen.current === -1 ? '从不' : `${lockscreen.current}s` }}</b></span>
      </div>

      <div class="preset-chips">
        <span
          v-for="preset in lockPresets"
          :key="preset.value"
          class="preset-chip"
          :class="{ active: lockscreen.inputSec === preset.value }"
          @click="lockscreen.inputSec = preset.value"
        >
          {{ preset.label }}
        </span>
      </div>

      <div class="field-group field-group-spacious">
        <div class="field-label">自定义时长（秒）</div>
        <div class="row-inputs">
          <el-input-number v-model="lockscreen.inputSec" class="flex-grow-input" :min="5" :max="1800" controls-position="right" :disabled="!isConnected" />
        </div>
      </div>

      <div class="action-row">
        <el-button type="primary" class="flex-grow-button" :disabled="!isConnected" @click="$emit('set-lockscreen')">修改</el-button>
      </div>

      <div class="divider section-divider" />

      <div class="field-label quick-actions-label">快捷按键</div>
      <div class="quick-row">
        <button
          v-for="action in quickActions"
          :key="action.label"
          class="quick-btn"
          :disabled="!isConnected"
          :title="action.label"
          @click="$emit('keyevent', action.code)"
        >
          <SmartIcon :name="action.icon" :color="action.color" :size="18" :show-background="false" :show-decoration="false" />
          <span>{{ action.label }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  lockscreen: {
    type: Object,
    required: true,
  },
  lockPresets: {
    type: Array,
    default: () => [],
  },
  quickActions: {
    type: Array,
    default: () => [],
  },
  isConnected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['set-lockscreen', 'keyevent']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.preset-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 6px;

  .preset-chip {
    padding: 5px 12px;
    font-size: 12px;
    border-radius: 20px;
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: all 0.2s;
    color: var(--color-text-secondary);
    user-select: none;

    &:hover {
      border-color: var(--color-primary);
      color: var(--color-primary);
    }

    &.active {
      background: rgba(var(--color-primary-rgb), 0.12);
      border-color: rgba(var(--color-primary-rgb), 0.22);
      color: var(--color-primary);
      font-weight: 500;
    }
  }
}

.quick-actions-label {
  margin-bottom: 10px;
}

.quick-row {
  display: flex;
  gap: 6px;

  .quick-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 4px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
    transition: all 0.2s;

    span {
      font-size: 10px;
      color: var(--color-text-muted);
    }

    &:hover:not(:disabled) {
      border-color: var(--color-primary);
      background: rgba(var(--color-primary-rgb), 0.06);

      span {
        color: var(--color-text-primary);
      }
    }

    &:disabled {
      opacity: 0.4;
      cursor: not-allowed;
    }
  }
}
</style>
