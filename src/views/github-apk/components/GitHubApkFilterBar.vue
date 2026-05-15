<template>
  <div class="filter-bar page-subtoolbar surface-card">
    <span class="filter-label">版本过滤:</span>
    <div class="filter-tags page-chip-group">
      <div
        v-for="filter in filters"
        :key="filter.value"
        class="filter-tag page-chip"
        :class="{ 'is-active': currentFilter === filter.value }"
        @click="$emit('change-filter', filter.value)"
      >
        {{ filter.label }}
      </div>
    </div>

    <div class="repo-summary">
      <div class="summary-pill">
        <span class="summary-key">仓库</span>
        <strong>{{ repo || '-' }}</strong>
      </div>
      <div class="summary-pill">
        <span class="summary-key">发行版</span>
        <strong>{{ releaseCount }}</strong>
      </div>
      <div class="summary-pill">
        <span class="summary-key">筛选后</span>
        <strong>{{ visibleCount }}</strong>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  filters: { type: Array, default: () => [] },
  currentFilter: { type: String, default: 'all' },
  repo: { type: String, default: '' },
  releaseCount: { type: Number, default: 0 },
  visibleCount: { type: Number, default: 0 },
});

defineEmits(['change-filter']);
</script>

<style lang="scss" scoped>
.filter-label {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-right: 12px;
}

.repo-summary {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.summary-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  padding: 0 14px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border);
  background: var(--surface-soft);
  color: var(--color-text-secondary);

  strong {
    color: var(--color-text-primary);
    font-size: 12px;
    font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace;
  }
}

.summary-key {
  font-size: 12px;
  color: var(--color-text-muted);
}
</style>
