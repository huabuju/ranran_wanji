<template>
  <div class="file-toolbar">
    <!-- 左：导航控制 -->
    <div class="nav-group">
      <button class="nav-btn" :disabled="!canGoUp" @click="$emit('go-up')" title="返回上级">
        <svg viewBox="0 0 20 20" fill="currentColor" width="16" height="16">
          <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd"/>
        </svg>
      </button>
      <!-- <button class="nav-btn" @click="$emit('refresh')" title="刷新">
        <svg viewBox="0 0 20 20" fill="currentColor" width="16" height="16" :class="{ spinning: loading }">
          <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd"/>
        </svg>
      </button> -->
    </div>

    <!-- 中：面包屑路径 -->
    <div class="breadcrumb-area">
      <div class="breadcrumb">
        <span
          v-for="(crumb, index) in breadcrumbs"
          :key="index"
          class="crumb"
        >
          <span
            class="crumb-text"
            :class="{ last: index === breadcrumbs.length - 1 }"
            @click="index < breadcrumbs.length - 1 && $emit('navigate', crumb.path)"
          >{{ crumb.label }}</span>
          <span v-if="index < breadcrumbs.length - 1" class="crumb-sep">›</span>
        </span>
      </div>
    </div>

    <!-- 快捷路径 -->
    <div class="quick-nav-group">
      <button
        v-for="nav in quickNavs"
        :key="nav.path"
        class="quick-btn"
        :class="{ active: currentPath === nav.path }"
        @click="$emit('navigate', nav.path)"
        :title="nav.path"
      >{{ nav.label }}</button>
    </div>

    <!-- 右：操作区 -->
    <div class="action-group">
      <!-- 搜索框 -->
      <el-input
        v-model="localSearch"
        placeholder="在此文件夹中搜索"
        clearable
        class="search-box page-search page-search--compact"
        @input="$emit('search', localSearch)"
      >
        <template #prefix>
          <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14" class="search-icon">
            <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd"/>
          </svg>
        </template>
      </el-input>

      <!-- 布局切换 -->
      <div class="layout-toggle">
        <button
          class="toggle-btn"
          :class="{ active: layoutMode === 'list' }"
          @click="$emit('set-layout', 'list')"
          title="列表视图"
        >
          <svg viewBox="0 0 20 20" fill="currentColor" width="15" height="15">
            <path fill-rule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"/>
          </svg>
        </button>
        <button
          class="toggle-btn"
          :class="{ active: layoutMode === 'grid' }"
          @click="$emit('set-layout', 'grid')"
          title="图标视图"
        >
          <svg viewBox="0 0 20 20" fill="currentColor" width="15" height="15">
            <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

const props = defineProps({
  currentPath: { type: String, default: '/sdcard' },
  layoutMode: { type: String, default: 'list' },
  loading: { type: Boolean, default: false },
});

defineEmits(['go-up', 'refresh', 'navigate', 'search', 'set-layout']);

const localSearch = ref('');

const canGoUp = computed(() => props.currentPath !== '/');

// 构建面包屑
const breadcrumbs = computed(() => {
  const parts = props.currentPath.split('/').filter(Boolean);
  const crumbs = [{ label: '根目录', path: '/' }];
  let accumulated = '';
  for (const part of parts) {
    accumulated += '/' + part;
    crumbs.push({ label: part, path: accumulated });
  }
  return crumbs;
});

const quickNavs = [
  { label: 'sdcard', path: '/sdcard' },
  { label: 'Download', path: '/sdcard/Download' },
  { label: 'DCIM', path: '/sdcard/DCIM' },
  { label: 'Android', path: '/sdcard/Android' },
  // { label: 'data', path: '/data' },
];
</script>

<style scoped>
.file-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: var(--bg-glass);
  backdrop-filter: var(--blur-glass);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-sm);
  flex-shrink: 0;
}

/* 导航按钮组 */
.nav-group {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.nav-btn {
  width: 30px;
  height: 30px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--surface-strong);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.15s;

  &:hover:not(:disabled) {
    background: var(--surface-soft);
    border-color: var(--border-strong);
    color: var(--color-text-primary);
  }

  &:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
}

.spinning {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 面包屑 */
.breadcrumb-area {
  flex: 1;
  min-width: 0;
  background: var(--surface-strong);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: 4px 12px;
  overflow: hidden;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
  overflow: hidden;
  font-size: 12px;
}

.crumb {
  display: flex;
  align-items: center;
  gap: 4px;
}

.crumb-text {
  color: var(--color-primary);
  cursor: pointer;
  transition: color 0.15s;
  padding: 1px 2px;
  border-radius: 3px;

  &:hover {
    color: var(--color-primary);
    background: rgba(var(--color-primary-rgb), 0.08);
  }

  &.last {
    color: var(--color-text-primary);
    font-weight: 600;
    cursor: default;
    &:hover { background: none; }
  }
}

.crumb-sep {
  color: var(--color-text-muted);
  font-size: 14px;
  line-height: 1;
}

/* 快捷导航 */
.quick-nav-group {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.quick-btn {
  padding: 3px 10px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--surface-strong);
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.15s;

  &:hover {
    background: var(--surface-soft);
    border-color: var(--border-strong);
    color: var(--color-text-primary);
  }

  &.active {
    background: rgba(var(--color-primary-rgb), 0.1);
    border-color: rgba(var(--color-primary-rgb), 0.28);
    color: var(--color-primary);
    font-weight: 600;
  }
}

/* 操作区 */
.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.search-icon {
  color: var(--color-text-muted);
}

/* 布局切换 */
.layout-toggle {
  display: flex;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--surface-strong);
}

.toggle-btn {
  width: 30px;
  height: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: all 0.15s;

  &:first-child { border-right: 1px solid var(--color-border); }

  &:hover { background: var(--surface-soft); color: var(--color-text-secondary); }
  &.active {
    background: var(--color-primary);
    color: var(--text-on-primary);
  }
}
</style>
