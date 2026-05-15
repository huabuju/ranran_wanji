<template>
  <transition name="topbar-menu-fade">
    <div v-if="show" class="menu-panel" @click.stop>
      <div class="menu-arrow"></div>
      <button
        v-for="item in items"
        :key="item.key"
        class="menu-item"
        type="button"
        @click="$emit('select', item)"
      >
        <SmartIcon
          :name="item.icon"
          :color="item.color || 'var(--color-text-secondary)'"
          :size="13"
          :show-background="false"
          :show-decoration="false"
        />
        <span>{{ item.label }}</span>
      </button>
    </div>
  </transition>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  show: {
    type: Boolean,
    default: false
  },
  items: {
    type: Array,
    default: () => []
  }
});

defineEmits(['select']);
</script>

<style lang="scss" scoped>
.menu-panel {
  position: absolute;
  top: calc(100% + 12px);
  right: 0;
  width: 188px;
  padding: 10px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.96);
  border: 1px solid rgba(226, 232, 240, 0.9);
  box-shadow: 0 24px 48px rgba(15, 23, 42, 0.14);
  backdrop-filter: blur(22px);
  z-index: 40;
}

.menu-arrow {
  position: absolute;
  top: -7px;
  right: 26px;
  width: 14px;
  height: 14px;
  background: rgba(255, 255, 255, 0.96);
  border-top: 1px solid rgba(226, 232, 240, 0.9);
  border-left: 1px solid rgba(226, 232, 240, 0.9);
  transform: rotate(45deg);
}

.menu-item {
  width: 100%;
  height: 42px;
  border: none;
  background: transparent;
  border-radius: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 14px;
  color: var(--color-text-primary);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover {
    background: rgba(var(--color-primary-rgb), 0.08);
    color: var(--color-primary);
    transform: translateX(2px);
  }
}

.topbar-menu-fade-enter-active,
.topbar-menu-fade-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.topbar-menu-fade-enter-from,
.topbar-menu-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
