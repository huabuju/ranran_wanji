<template>
  <transition name="fade-scale">
    <div v-show="show" class="glass-modal-mask" @click.self="handleMaskClick">
      <div
        class="glass-modal-container"
        :class="containerClass"
        :style="{ '--glass-modal-max-width': maxWidth }"
      >
        <!-- Slot for all content -->
        <slot></slot>
      </div>
    </div>
  </transition>
</template>

<script setup>
const props = defineProps({
  show: {
    type: Boolean,
    default: false
  },
  maxWidth: {
    type: String,
    default: '420px'
  },
  clickToClose: {
    type: Boolean,
    default: false
  },
  containerClass: {
    type: [String, Array, Object],
    default: ''
  }
});

const emit = defineEmits(['close']);

function handleMaskClick() {
  if (props.clickToClose) {
    emit('close');
  }
}
</script>

<style lang="scss" scoped>
.glass-modal-mask {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: var(--overlay-mask);
  backdrop-filter: blur(8px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.glass-modal-container {
  position: relative;
  width: 100%;
  max-width: var(--glass-modal-max-width);
  background: var(--app-dialog-bg, var(--surface-elevated));
  backdrop-filter: blur(20px);
  border: 1px solid var(--app-dialog-border, var(--border-strong));
  border-radius: var(--app-dialog-radius, 24px);
  box-shadow: var(--app-dialog-shadow, var(--shadow-card-hover));
  overflow: hidden;
}

/* Animations */
.fade-scale-enter-active, .fade-scale-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.fade-scale-enter-from, .fade-scale-leave-to {
  opacity: 0;
  transform: scale(0.9) translateY(20px);
}
</style>
