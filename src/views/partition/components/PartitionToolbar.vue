<template>
  <div class="top-action-bar page-toolbar surface-card">
    <div class="left-actions page-toolbar-left">
      <el-button type="danger" class="scan-btn" @click="$emit('scan')">刷新分区</el-button>
      <el-input v-model="searchModel" placeholder="搜索分区名称或路径..." clearable class="page-search">
        <template #prefix>
          <SmartIcon name="search" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
        </template>
      </el-input>
    </div>
    <div class="right-actions page-toolbar-right">
      <div class="stats-group page-stats">
        <div class="stat-item page-stat">
          <SmartIcon name="copy" color="var(--brand-primary-strong)" :size="12" />
          <span class="stat-label">全部分区</span>
        </div>
        <div class="stat-item blue page-stat">
          <span class="stat-value page-stat-value">{{ total }}</span>
          <span class="stat-label">总计</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  searchQuery: { type: String, default: '' },
  total: { type: Number, default: 0 },
});

const emit = defineEmits(['update:search-query', 'scan']);

const searchModel = computed({
  get: () => props.searchQuery,
  set: (value) => emit('update:search-query', value),
});
</script>

<style lang="scss" scoped>
.scan-btn {
  background-color: var(--color-danger);
  border-color: var(--color-danger);
  font-weight: 600;
  border-radius: var(--radius-full);
  padding: 8px 16px;
  transition: all 0.3s ease;

  &:hover {
    background-color: var(--color-danger);
    border-color: var(--color-danger);
    transform: translateY(-1px);
    box-shadow: 0 8px 18px -8px rgba(var(--color-danger-rgb), 0.7);
  }
}
</style>
