<template>
  <div 
    class="smart-icon-container" 
    :style="iconVars"
    :class="{ 'has-bg': showBackground }"
  >
    <svg 
      viewBox="0 0 24 24" 
      fill="none" 
      xmlns="http://www.w3.org/2000/svg"
      class="smart-icon-svg"
    >
      <!-- 主图标路径 -->
      <path 
        v-for="(d, index) in paths" 
        :key="index"
        :d="d" 
        stroke="currentColor" 
        stroke-width="1.8" 
        stroke-linecap="round" 
        stroke-linejoin="round"
      />
      
      <!-- 装饰元素：小圆点 -->
      <circle 
        v-if="showDecoration"
        cx="19" cy="5" r="1.5" 
        fill="currentColor" 
        fill-opacity="0.6"
      />
      <circle 
        v-if="showDecoration"
        cx="4" cy="18" r="1" 
        fill="currentColor" 
        fill-opacity="0.4"
      />
    </svg>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { iconLibrary } from '@/config/icons';

const props = defineProps({
  name: {
    type: String,
    required: true
  },
  color: {
    type: String,
    default: 'var(--color-info)'
  },
  size: {
    type: [Number, String],
    default: 24
  },
  showBackground: {
    type: Boolean,
    default: true
  },
  showDecoration: {
    type: Boolean,
    default: true
  }
});

const paths = computed(() => iconLibrary[props.name] || []);

const iconVars = computed(() => {
  const containerSizeValue = typeof props.size === 'number' ? `${props.size * 1.6}px` : props.size;
  const sizeValue = typeof props.size === 'number' ? `${props.size}px` : props.size;
  return {
    '--smart-icon-container-size': containerSizeValue,
    '--smart-icon-size': sizeValue,
    '--smart-icon-color': props.color,
    '--smart-icon-bg': props.showBackground
      ? `color-mix(in srgb, ${props.color} 14%, transparent)`
      : 'transparent'
  };
});
</script>

<style scoped>
.smart-icon-container {
  width: var(--smart-icon-container-size);
  height: var(--smart-icon-container-size);
  color: var(--smart-icon-color);
  background: var(--smart-icon-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px; /* Squircle appearance */
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.smart-icon-svg {
  width: var(--smart-icon-size);
  height: var(--smart-icon-size);
  transition: transform 0.3s ease;
}

.smart-icon-container:hover .smart-icon-svg {
  transform: scale(1.1);
}

.has-bg {
  box-shadow: inset 0 0 0 1px var(--border-strong);
}
</style>
