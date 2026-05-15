<template>
  <GlassModal :show="show" maxWidth="320px">
    <div class="loading-content">
      <!-- Spinner & Icon Slot -->
      <div class="spinner-container">
        <div class="outer-ring"></div>
        <div class="icon-slot">
          <slot name="icon">
            <el-icon class="default-icon"><Loading /></el-icon>
          </slot>
        </div>
      </div>
      
      <!-- Text Content -->
      <div class="text-group">
        <h3 v-if="title" class="title">{{ title }}</h3>
        <p v-if="description" class="description">{{ description }}</p>
      </div>
    </div>
  </GlassModal>
</template>

<script setup>
import { Loading } from '@element-plus/icons-vue';
import GlassModal from './GlassModal.vue';

defineProps({
  show: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: ''
  },
  description: {
    type: String,
    default: ''
  }
});
</script>

<style lang="scss" scoped>
.loading-content {
  padding: 40px 30px;
}

.spinner-container {
  position: relative;
  width: 70px;
  height: 70px;
  margin: 0 auto 24px;
  display: flex;
  align-items: center;
  justify-content: center;

  .outer-ring {
    position: absolute;
    width: 100%;
    height: 100%;
    border: 3px solid transparent;
    border-top: 3px solid var(--brand-primary-strong);
    border-right: 3px solid var(--brand-violet);
    border-radius: 50%;
    animation: rotate 1.2s linear infinite;
  }

  .icon-slot {
    font-size: 28px;
    color: var(--color-primary);
    
    .default-icon {
      animation: pulse 1.5s ease-in-out infinite;
    }
  }
}

.text-group {
  .title {
    font-size: 18px;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 8px;
    letter-spacing: -0.01em;
  }

  .description {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
    opacity: 0.9;
  }
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.8; }
  50% { transform: scale(1.15); opacity: 1; }
}
</style>
