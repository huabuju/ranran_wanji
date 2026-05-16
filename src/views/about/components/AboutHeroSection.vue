<template>
  <div class="about-header section-fade-in">
    <div class="logo-wrapper">
      <SmartIcon
        name="logo"
        color="white"
        :size="48"
        :show-background="false"
        :show-decoration="true"
      />
    </div>
    <h1 class="app-name">{{ title }}</h1>
    <p class="app-subtitle">
      <span class="version">v{{ version }}</span>
      <template v-if="buildTime">
        <span class="divider">•</span>
        <span class="build-time">打包于 {{ buildTime }}</span>
      </template>
      <span class="divider">•</span>
      <span class="description">{{ description }}</span>
    </p>
    <!-- <p v-if="runtimePath" class="app-subtitle" :title="runtimePathFull || runtimePath">
      <span class="description">Runtime Path</span>
      <span class="divider">·</span>
      <span class="description">{{ runtimePath }}</span>
    </p> -->
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  title: { type: String, required: true },
  version: { type: String, required: true },
  buildTime: { type: String, default: '' },
  description: { type: String, required: true },
  runtimePath: { type: String, default: '' },
  runtimePathFull: { type: String, default: '' },
});
</script>

<style lang="scss" scoped>
.about-header {
  text-align: center;
  margin-bottom: 48px;
}

.logo-wrapper {
  width: 80px;
  height: 80px;
  margin: 0 auto 24px;
  background: var(--primary-gradient);
  border-radius: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--primary-shadow-card);
  transition: transform 0.3s ease;

  &:hover {
    transform: scale(1.05) rotate(5deg);
  }
}

.app-name {
  font-size: 32px;
  font-weight: 800;
  color: var(--color-text-primary);
  margin-bottom: 8px;
  letter-spacing: -0.5px;
}

.app-subtitle {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 3px;
  padding: 2px 0;
  box-sizing: border-box;
  color: var(--color-text-secondary);
  font-size: 14px;
}

.divider {
  opacity: 0.3;
}

</style>
