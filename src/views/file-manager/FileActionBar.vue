<template>
  <div class="action-bar page-toolbar surface-card">
    <div class="left-ops page-toolbar-left">
      <el-button size="small" type="primary" @click="$emit('push')">
        <template #icon>
          <svg viewBox="0 0 16 16" fill="currentColor" width="12" height="12"><path fill-rule="evenodd" d="M8 2a.5.5 0 01.5.5v6.793l2.146-2.147a.5.5 0 01.708.708l-3 3a.5.5 0 01-.708 0l-3-3a.5.5 0 01.708-.708L7.5 9.293V2.5A.5.5 0 018 2z" transform="rotate(180 8 8)"/><path d="M.5 14.5a.5.5 0 011 0v.5h13v-.5a.5.5 0 011 0V15a.5.5 0 01-.5.5h-14A.5.5 0 01.5 15v-.5z"/></svg>
        </template>
        上传文件
      </el-button>
      <el-button size="small" @click="$emit('push-dir')">上传文件夹</el-button>
      <el-button size="small" @click="$emit('mkdir')">
        <template #icon>
          <svg viewBox="0 0 16 16" fill="currentColor" width="12" height="12"><path d="M.54 3.87L.5 3a2 2 0 012-2h3.672a2 2 0 011.414.586l.828.828A2 2 0 009.828 3h3.982a2 2 0 011.992 2.181l-.637 7A2 2 0 0113.174 14H2.826a2 2 0 01-1.991-1.819l-.637-7a1.99 1.99 0 01.342-1.31zM8 6.993c.016-.346.216-.54.501-.54.286 0 .478.194.478.54V8h1.026c.329 0 .52.198.52.498 0 .3-.191.498-.52.498H8.98v1.006c0 .328-.192.526-.501.526-.31 0-.481-.202-.481-.526V8.996H6.974c-.329 0-.52-.198-.52-.498 0-.3.191-.498.52-.498H7.999V6.993z"/></svg>
        </template>
        新建文件夹
      </el-button>
    </div>

    <div class="right-info page-toolbar-right">
      <span class="file-count">{{ displayedFilesCount }} 项</span>
      <span v-if="selectedCount > 0" class="selected-count">已选 {{ selectedCount }} 项</span>
      <div v-if="storageInfo" class="storage-bar-wrap">
        <span class="storage-text">存储: {{ formatSize(storageInfo.used_bytes) }} / {{ formatSize(storageInfo.total_bytes) }}</span>
        <div class="storage-bar">
          <div class="storage-fill" :style="{ '--storage-fill-width': storageInfo.use_percent + '%' }" :class="{ warning: storageInfo.use_percent > 80 }" />
        </div>
        <span class="storage-pct">{{ storageInfo.use_percent }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  displayedFilesCount: { type: Number, default: 0 },
  selectedCount: { type: Number, default: 0 },
  storageInfo: { type: Object, default: null },
  formatSize: { type: Function, required: true },
});

defineEmits(['push', 'push-dir', 'mkdir']);
</script>

<style lang="scss" scoped>
.action-bar {
  flex-shrink: 0;
}

.left-ops {
  gap: 8px;
}

.right-info {
  gap: 12px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.file-count {
  color: var(--color-text-secondary);
}

.selected-count {
  color: var(--color-primary);
  font-weight: 600;
  background: rgba(var(--color-primary-rgb), 0.08);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.storage-bar-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.storage-text {
  color: var(--color-text-secondary);
}

.storage-bar {
  width: 80px;
  height: 5px;
  background: var(--color-border);
  border-radius: 3px;
  overflow: hidden;
}

.storage-fill {
  width: var(--storage-fill-width);
  height: 100%;
  background: var(--color-primary);
  border-radius: 3px;
  transition: width 0.5s ease;

  &.warning {
    background: var(--color-warning);
  }
}

.storage-pct {
  color: var(--color-text-muted);
}
</style>
