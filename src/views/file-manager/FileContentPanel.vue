<template>
  <div class="content-panel page-table-card surface-card-strong" v-loading="loading" element-loading-text="正在读取文件列表...">
    <div v-if="errMsg" class="error-state">
      <svg viewBox="0 0 24 24" fill="none" width="40" height="40">
        <circle cx="12" cy="12" r="10" stroke="var(--color-danger)" stroke-width="1.5" />
        <path d="M12 7v5M12 16v1" stroke="var(--color-danger)" stroke-width="2" stroke-linecap="round" />
      </svg>
      <p>{{ errMsg }}</p>
      <el-button size="small" @click="$emit('refresh')">重试</el-button>
    </div>

    <FileListView
      v-else-if="layoutMode === 'list'"
      :files="displayedFiles"
      :selected-paths="selectedPaths"
      @navigate="$emit('navigate', $event)"
      @select="handleSelect"
      @context-menu="handleContextMenu"
    />

    <FileGridView
      v-else
      :files="displayedFiles"
      :selected-paths="selectedPaths"
      @navigate="$emit('navigate', $event)"
      @select="handleSelect"
      @context-menu="handleContextMenu"
    />
  </div>
</template>

<script setup>
import FileGridView from './FileGridView.vue';
import FileListView from './FileListView.vue';

defineProps({
  loading: { type: Boolean, default: false },
  errMsg: { type: String, default: '' },
  layoutMode: { type: String, default: 'grid' },
  displayedFiles: { type: Array, default: () => [] },
  selectedPaths: { type: Object, default: () => new Set() },
});

const emit = defineEmits(['refresh', 'navigate', 'select', 'context-menu']);

function handleSelect(...args) {
  emit('select', ...args);
}

function handleContextMenu(...args) {
  emit('context-menu', ...args);
}
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.content-panel {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}

.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  color: var(--color-text-muted);
  font-size: 13px;
  padding: 40px;
  text-align: center;
}
</style>
