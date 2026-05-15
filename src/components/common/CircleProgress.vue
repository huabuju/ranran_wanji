<template>
  <div class="circle-progress" :style="progressVars">
    <el-progress
      type="circle"
      :percentage="value"
      :width="size"
      :stroke-width="strokeWidth"
      :color="color"
    >
      <template #default="{ percentage }">
        <span class="circle-value">
          {{ percentage }}%
        </span>
      </template>
    </el-progress>
    <div class="circle-label">{{ label }}</div>
    <div class="circle-subtext">{{ subtext }}</div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  value: { type: Number, default: 0 },   // 0-100
  size: { type: Number, default: 80 },
  strokeWidth: { type: Number, default: 7 },
  color: { type: String, default: 'var(--color-primary)' },
  bgColor: { type: String, default: 'var(--surface-soft)' },
  label: { type: String, default: '' },
  subtext: { type: String, default: '' },
});

const fontSize = computed(() => Math.round(props.size * 0.18));
const progressVars = computed(() => ({
  '--circle-font-size': `${fontSize.value}px`,
  '--circle-value-color': props.color,
}));
</script>

<style lang="scss" scoped>
.circle-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;

  .circle-label {
    font-size: 12px;
    color: var(--color-text-secondary);
    font-weight: 400;
    margin-top: 2px;
  }

  .circle-subtext {
    font-size: 11px;
    color: var(--color-text-muted);
    margin-top: -4px;
  }

  .circle-value {
    font-size: var(--circle-font-size);
    color: var(--circle-value-color);
    font-weight: 700;
  }
}
</style>
