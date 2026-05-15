<template>
  <div class="dashboard-card status-top-bar page-toolbar surface-card">
    <div class="top-left page-toolbar-left">
      <el-input v-model="searchModel" placeholder="搜索键名或取值..." clearable class="page-search">
        <template #prefix>
          <SmartIcon name="search" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
        </template>
      </el-input>
      <el-button class="action-btn" :loading="loading" @click="$emit('refresh')">
        <span>刷新列表</span>
      </el-button>
    </div>

    <div class="top-right page-toolbar-right">
      <el-button type="primary" class="add-btn" @click="$emit('add')">
        <span>新增属性</span>
      </el-button>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  searchQuery: { type: String, default: '' },
  loading: { type: Boolean, default: false },
});

const emit = defineEmits(['update:search-query', 'refresh', 'add']);

const searchModel = computed({
  get: () => props.searchQuery,
  set: (value) => emit('update:search-query', value),
});
</script>

<style lang="scss" scoped>
.dashboard-card {
  min-width: 0;
}

.status-top-bar {
  flex-shrink: 0;
}

.top-left {
  .action-btn {
    background: var(--color-primary);
    border: none;
    color: white;
    border-radius: var(--radius-full);
    gap: 6px;

    &:hover {
      background: var(--color-primary-hover);
    }
  }
}

.top-right {
  display: flex;
  align-items: center;
  gap: 20px;

  .add-btn {
    border-radius: var(--radius-full);
    gap: 6px;
  }
}
</style>
