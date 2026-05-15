<template>
  <div class="top-action-bar page-toolbar surface-card">
    <div class="left-actions page-toolbar-left">
      <el-button type="primary" class="install-btn" @click="$emit('install')">安装 APK</el-button>
      <el-button @click="$emit('refresh')">刷新列表</el-button>
    </div>

    <div class="right-actions page-toolbar-right">
      <el-input v-model="searchModel" placeholder="搜索包名/应用名/UID" clearable class="page-search">
        <template #prefix>
          <SmartIcon name="search" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
        </template>
      </el-input>
      <div class="stats-group page-stats">
        <div class="stat-item page-stat">
          <SmartIcon name="system" color="var(--color-info)" :size="14" :show-background="false" :show-decoration="false" />
          <span class="stat-label">已安装应用</span>
        </div>
        <div class="stat-item blue page-stat">
          <span class="stat-value page-stat-value">{{ total }}</span>
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

const emit = defineEmits(['update:search-query', 'install', 'refresh']);

const searchModel = computed({
  get: () => props.searchQuery,
  set: (value) => emit('update:search-query', value),
});
</script>
