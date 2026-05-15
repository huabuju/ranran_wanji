<template>
  <!-- 网格图标视图 -->
  <div class="file-grid-view" @click.self="$emit('select', new Set(), null)">
    <div
      v-for="file in files"
      :key="file.path"
      class="grid-item"
      :class="{ selected: selectedPaths.has(file.path) }"
      @click.exact="$emit('select', new Set([file.path]), file)"
      @click.ctrl="toggleSelect(file)"
      @dblclick="file.is_dir ? $emit('navigate', file.path) : null"
      @contextmenu.prevent.stop="onContextMenu($event, file)"
    >
      <div class="icon-wrap">
        <FileIcon :name="file.name" :is-dir="file.is_dir" :size="64" :is-grid="true" />
      </div>
      <div class="item-name" :title="file.name">{{ file.name }}</div>
    </div>

    <!-- 空状态 -->
    <div v-if="files.length === 0" class="empty-state">
      <svg viewBox="0 0 64 64" fill="none" width="56" height="56" class="empty-illustration">
        <rect x="8" y="14" width="48" height="40" rx="4" class="empty-frame" stroke-width="2"/>
        <path d="M8 22H56" class="empty-line" stroke-width="1.5"/>
        <circle cx="32" cy="40" r="8" class="empty-dot"/>
      </svg>
      <p>此文件夹为空</p>
    </div>
  </div>
</template>

<script setup>
import FileIcon from './FileIcon.vue';

const props = defineProps({
  files: { type: Array, default: () => [] },
  selectedPaths: { type: Set, default: () => new Set() },
});
const emit = defineEmits(['navigate', 'select', 'context-menu']);

function toggleSelect(file) {
  const newSet = new Set(props.selectedPaths);
  if (newSet.has(file.path)) newSet.delete(file.path);
  else newSet.add(file.path);
  emit('select', newSet, file);
}

function onContextMenu(event, file) {
  if (!props.selectedPaths.has(file.path)) {
    emit('select', new Set([file.path]), file);
  }
  emit('context-menu', event, file);
}
</script>

<style scoped>
.file-grid-view {
  display: grid;
  grid-template-columns: repeat(auto-fill, 108px);
  justify-content: space-between;
  align-content: flex-start;
  gap: 12px;
  padding: 16px;
  overflow-y: auto;
  height: 100%;
  background: transparent;
}

.grid-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px 6px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, box-shadow 0.15s;
  border: 2px solid transparent;

  &:hover {
    background: var(--table-row-hover);
    border-color: rgba(var(--color-primary-rgb), 0.12);
    box-shadow: 0 8px 18px rgba(15, 23, 42, 0.06);
  }

  &.selected {
    background: rgba(var(--color-primary-rgb), 0.12);
    border-color: rgba(var(--color-primary-rgb), 0.24);
  }
}

.icon-wrap {
  width: 70px;
  height: 70px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.item-name {
  width: 100%;
  text-align: center;
  font-size: 11px;
  color: var(--color-text-primary);
  line-height: 1.4;
  word-break: break-all;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.empty-state {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.empty-illustration {
  .empty-frame {
    fill: var(--surface-soft);
    stroke: var(--color-border);
  }

  .empty-line {
    stroke: var(--color-border);
  }

  .empty-dot {
    fill: var(--color-divider);
  }
}
</style>
