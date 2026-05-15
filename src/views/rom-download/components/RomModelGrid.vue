<template>
  <div class="models-container">
    <div class="model-grid">
      <div v-for="model in models" :key="model.key" class="model-card" @click="$emit('select-model', model)">
        <div class="model-header">
          <SmartIcon name="device" color="var(--color-primary)" :size="16" :show-background="false" :show-decoration="false" />
          <span class="model-name">{{ model.name }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  models: { type: Array, default: () => [] },
});

defineEmits(['select-model']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.models-container { display: flex; flex-direction: column; }
.model-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
.model-card {
  background: var(--bg-glass);
  backdrop-filter: var(--blur-glass);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 16px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: var(--shadow-card);
  @include pageCard.overview-main-card-hoverable(var(--bg-glass));
}
.model-header { display: flex; align-items: center; gap: 8px; }
.model-name { font-size: 14px; font-weight: 700; color: var(--color-text-primary); }
</style>
