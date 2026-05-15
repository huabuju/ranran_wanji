<template>
  <div class="tech-tags section-fade-in">
    <span v-for="tag in tags" :key="tag" class="tag">{{ tag }}</span>
  </div>
</template>

<script setup>
defineProps({
  tags: { type: Array, default: () => [] },
});
</script>

<style lang="scss" scoped>
.tech-tags {
  display: flex;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 40px;
}

.tag {
  padding: 4px 12px;
  background: var(--surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 20px;
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 500;
}
</style>
