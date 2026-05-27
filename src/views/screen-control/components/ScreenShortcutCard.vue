<template>
  <el-card class="custom-card shortcut-card" shadow="hover">
    <template #header>
      <div class="card-header flex-y-center">
        <div class="header-icon">
          <SmartIcon name="bolt" color="var(--color-warning)" :size="16" :show-background="false" :show-decoration="false" />
        </div>
        <div class="header-title">
          <span class="title-main">便捷控制</span>
          <span class="title-sub">虚拟按键 · 快速操作</span>
        </div>
        <div class="status-chip" :class="{ 'is-live': isStreaming }">
          <span class="chip-dot" />
          <span>{{ isStreaming ? '可用' : '待命' }}</span>
        </div>
      </div>
    </template>

    <div class="shortcut-grid">
      <button
        v-for="action in actions"
        :key="action.code"
        type="button"
        class="key-btn"
        :class="{ 'is-disabled': !isStreaming }"
        :title="action.title"
        :disabled="!isStreaming"
        :style="{ '--key-accent': action.color }"
        @click="$emit('keyevent', action.code)"
      >
        <span class="key-bg" aria-hidden="true" />
        <span class="key-icon">
          <SmartIcon :name="action.icon" :color="action.color" :size="22" :show-background="false" :show-decoration="false" />
        </span>
        <span class="key-label">{{ action.title }}</span>
        <span class="key-ripple" aria-hidden="true" />
      </button>
    </div>
  </el-card>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  actions: { type: Array, default: () => [] },
  isStreaming: { type: Boolean, default: false },
});

defineEmits(['keyevent']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.shortcut-card {
  :deep(.el-card__body) {
    padding: 18px 22px 20px;
  }
}

.header-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 9px;
  background: linear-gradient(135deg, rgba(var(--color-warning-rgb), 0.18), rgba(var(--color-warning-rgb), 0.06));
  border: 1px solid rgba(var(--color-warning-rgb), 0.22);
  margin-right: 10px;
}

.header-title {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
  flex: 1;

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

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  background: var(--surface-chip);
  border: 1px solid var(--border-soft);
  color: var(--color-text-muted);
  transition: all 0.3s ease;

  .chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-text-muted);
  }

  &.is-live {
    color: var(--color-success);
    background: var(--success-soft);
    border-color: rgba(var(--color-success-rgb), 0.32);

    .chip-dot {
      background: var(--color-success);
      box-shadow: 0 0 8px rgba(var(--color-success-rgb), 0.7);
      animation: chip-pulse 1.6s ease-in-out infinite;
    }
  }
}

.shortcut-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(96px, 1fr));
  gap: 12px;
}

.key-btn {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px 10px 12px;
  border: 1px solid var(--border-soft);
  border-radius: 14px;
  background: var(--surface-soft);
  cursor: pointer;
  overflow: hidden;
  isolation: isolate;
  transition:
    transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1),
    border-color 0.3s ease,
    box-shadow 0.3s ease,
    background 0.3s ease;

  &:hover:not(:disabled) {
    transform: translateY(-3px);
    border-color: color-mix(in srgb, var(--key-accent) 35%, transparent);
    background: var(--surface-strong);
    box-shadow: 0 14px 28px -16px color-mix(in srgb, var(--key-accent) 50%, transparent);

    .key-bg { opacity: 1; }
    .key-icon {
      transform: translateY(-1px) scale(1.06);
    }
    .key-label {
      color: var(--key-accent);
    }
  }

  &:active:not(:disabled) {
    transform: translateY(-1px);
    transition-duration: 0.12s;

    .key-ripple {
      animation: key-ripple 0.55s ease-out;
    }
  }

  &.is-disabled {
    cursor: not-allowed;
    opacity: 0.55;
    filter: saturate(0.4);
  }
}

.key-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 50% 0%, color-mix(in srgb, var(--key-accent) 22%, transparent), transparent 65%),
    linear-gradient(180deg, color-mix(in srgb, var(--key-accent) 6%, transparent), transparent 70%);
  opacity: 0;
  transition: opacity 0.4s ease;
  pointer-events: none;
}

.key-icon {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 12px;
  background: var(--surface-chip);
  border: 1px solid var(--border-soft);
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.key-label {
  position: relative;
  z-index: 1;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--color-text-secondary);
  transition: color 0.3s ease;
}

.key-ripple {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  background: radial-gradient(circle at center, color-mix(in srgb, var(--key-accent) 32%, transparent) 0%, transparent 60%);
  opacity: 0;
  transform: scale(0.4);
}

@keyframes key-ripple {
  0% { opacity: 0.7; transform: scale(0.4); }
  100% { opacity: 0; transform: scale(1.4); }
}

@keyframes chip-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(var(--color-success-rgb), 0.5); }
  50% { box-shadow: 0 0 0 6px rgba(var(--color-success-rgb), 0); }
}
</style>