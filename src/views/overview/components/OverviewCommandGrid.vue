<template>
  <div class="toolbar-section dashboard-card">
    <div class="command-grid">
      <button
        v-for="opt in options"
        :key="opt.label"
        class="command-card"
        @click="$emit('reboot', opt)"
        :disabled="!isConnected"
        :class="{ 'is-disabled': !isConnected }"
        :style="getCommandVars(opt)"
      >
        <div class="command-icon">
          <SmartIcon :name="opt.icon" :color="!isConnected ? 'var(--color-text-muted)' : opt.color" :size="18" />
        </div>
        <div class="command-copy">
          <strong>{{ opt.label }}</strong>
          <span>{{ opt.desc || '立即执行设备控制命令' }}</span>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineEmits(['reboot']);

defineProps({
  options: {
    type: Array,
    default: () => [],
  },
  isConnected: {
    type: Boolean,
    default: false,
  },
});

const getCommandVars = (option) => ({
  '--command-accent': option.color,
  '--command-hover-bg': option.hoverBg,
});
</script>

<style lang="scss" scoped>
@use './shared.scss' as *;

.dashboard-card {
  @include overview-dashboard-card-shell;
}

.toolbar-section {
  padding: 18px 20px 20px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  flex-shrink: 0;
}

.command-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 14px;
}

.command-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px;
  @include overview-interactive-panel(
    18px,
    color-mix(in srgb, var(--color-border) 75%, transparent),
    linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated)),
    radial-gradient(circle at top left, color-mix(in srgb, var(--command-accent) 20%, transparent), transparent 70%),
    (transform 0.22s ease, box-shadow 0.22s ease, border-color 0.22s ease, background 0.22s ease)
  );
  cursor: pointer;
  text-align: left;
}

.command-card:hover:not(.is-disabled) {
  transform: translateY(-3px);
  border-color: color-mix(in srgb, var(--command-accent) 45%, var(--border-strong) 20%);
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.12), 0 4px 10px color-mix(in srgb, var(--command-accent) 20%, transparent);
}

.command-card.is-disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.command-icon {
  @include overview-icon-box-elevated(
    44px,
    14px,
    color-mix(in srgb, var(--command-accent) 12%, transparent),
    inherit
  );
}

.command-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.command-copy strong {
  font-size: 14px;
  color: var(--color-text-primary);
}

.command-copy span {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

@media (max-width: 1200px) {
  .command-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .command-grid {
    grid-template-columns: 1fr;
  }
}
</style>
