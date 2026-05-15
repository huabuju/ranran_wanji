<template>
  <div class="tab-scroller dashboard-card page-tabbar surface-card">
    <div
      v-for="tab in tabs"
      :key="tab.value"
      class="tab-item"
      :class="{ active: currentTab === tab.value }"
      @click="$emit('change', tab.value)"
    >
      <SmartIcon :name="tab.icon" :size="16" :color="currentTab === tab.value ? 'var(--color-info)' : 'var(--color-text-secondary)'" />
      <span class="tab-label">{{ tab.label }}</span>
      <span v-if="currentTab === tab.value" class="count-badge">{{ count }}</span>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  tabs: { type: Array, default: () => [] },
  currentTab: { type: String, default: 'getprop' },
  count: { type: Number, default: 0 },
});

defineEmits(['change']);
</script>

<style lang="scss" scoped>
.dashboard-card {
  min-width: 0;
}

.tab-scroller {
  flex-shrink: 0;
}

.tab-item {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 10px;
  cursor: pointer;
  border-radius: 12px;
  transition: all 0.2s;
  user-select: none;

  .tab-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .count-badge {
    background: var(--color-primary);
    color: white;
    font-size: 10px;
    padding: 0 6px;
    border-radius: 10px;
    height: 16px;
    display: flex;
    align-items: center;
  }

  &:hover {
    background: rgba(var(--color-primary-rgb), 0.06);
  }

  &.active {
    background: rgba(var(--color-primary-rgb), 0.1);
    box-shadow: inset 0 0 0 1px rgba(var(--color-primary-rgb), 0.16);

    .tab-label {
      color: var(--color-text-primary);
      font-weight: 600;
    }
  }
}
</style>
