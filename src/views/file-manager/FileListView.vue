<template>
  <!-- 列表视图 -->
  <div class="file-list-view">
    <!-- 表头 -->
    <div class="list-header">
      <div class="col-name" @click="toggleSort('name')">
        <span>名称</span>
        <span class="sort-arrow" v-if="sortKey === 'name'">{{ sortAsc ? '↑' : '↓' }}</span>
      </div>
      <div class="col-date" @click="toggleSort('modified')">
        <span>修改日期</span>
        <span class="sort-arrow" v-if="sortKey === 'modified'">{{ sortAsc ? '↑' : '↓' }}</span>
      </div>
      <div class="col-type">类型</div>
      <div class="col-size" @click="toggleSort('size')">
        <span>大小</span>
        <span class="sort-arrow" v-if="sortKey === 'size'">{{ sortAsc ? '↑' : '↓' }}</span>
      </div>
    </div>

    <!-- 文件行 -->
    <div class="list-body" ref="listBodyRef">
      <div
        v-for="file in sortedFiles"
        :key="file.path"
        class="list-row"
        :class="{ selected: selectedPaths.has(file.path), 'is-dir': file.is_dir }"
        @click.exact="selectOne(file)"
        @click.ctrl="toggleSelect(file)"
        @dblclick="file.is_dir ? $emit('navigate', file.path) : null"
        @contextmenu.prevent.stop="showContextMenu($event, file)"
      >
        <!-- 图标 + 名称 -->
        <div class="col-name">
          <FileIcon :name="file.name" :is-dir="file.is_dir" :size="20" />
          <span class="file-name" :title="file.name">{{ file.name }}</span>
          <span v-if="file.is_symlink" class="symlink-badge">快捷方式</span>
        </div>
        <!-- 修改日期 -->
        <div class="col-date">{{ file.modified }}</div>
        <!-- 类型 -->
        <div class="col-type">{{ getFileType(file) }}</div>
        <!-- 大小 -->
        <div class="col-size">{{ file.is_dir ? '' : formatSize(file.size) }}</div>
      </div>

      <!-- 空状态 -->
      <div v-if="sortedFiles.length === 0" class="empty-state">
        <svg viewBox="0 0 64 64" fill="none" width="48" height="48" class="empty-illustration">
          <rect x="8" y="12" width="48" height="40" rx="4" class="empty-frame" stroke-width="2"/>
          <path d="M8 20H56" class="empty-line" stroke-width="1.5"/>
          <circle cx="32" cy="38" r="8" class="empty-dot"/>
        </svg>
        <p>此文件夹为空</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import FileIcon from './FileIcon.vue';

const props = defineProps({
  files: { type: Array, default: () => [] },
  selectedPaths: { type: Set, default: () => new Set() },
});
const emit = defineEmits(['navigate', 'select', 'context-menu']);

const listBodyRef = ref(null);
const sortKey = ref('name');
const sortAsc = ref(true);

function toggleSort(key) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value;
  } else {
    sortKey.value = key;
    sortAsc.value = true;
  }
}

const sortedFiles = computed(() => {
  const list = [...props.files];
  list.sort((a, b) => {
    // 目录优先
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
    let av = a[sortKey.value] ?? '';
    let bv = b[sortKey.value] ?? '';
    if (sortKey.value === 'size') {
      av = Number(av); bv = Number(bv);
    } else {
      av = String(av).toLowerCase();
      bv = String(bv).toLowerCase();
    }
    if (av < bv) return sortAsc.value ? -1 : 1;
    if (av > bv) return sortAsc.value ? 1 : -1;
    return 0;
  });
  return list;
});

function selectOne(file) {
  emit('select', new Set([file.path]), file);
}

function toggleSelect(file) {
  const newSet = new Set(props.selectedPaths);
  if (newSet.has(file.path)) newSet.delete(file.path);
  else newSet.add(file.path);
  emit('select', newSet, file);
}

function showContextMenu(event, file) {
  if (!props.selectedPaths.has(file.path)) {
    emit('select', new Set([file.path]), file);
  }
  emit('context-menu', event, file);
}

function formatSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

const TYPE_MAP = {
  // 文件夹
  dir: '文件夹',
  // 图片
  jpg: 'JPEG 图片', jpeg: 'JPEG 图片', png: 'PNG 图片',
  gif: 'GIF 动图', webp: 'WebP 图片', svg: 'SVG 图片', heic: 'HEIC 图片',
  // 视频
  mp4: 'MP4 视频', mkv: 'MKV 视频', avi: 'AVI 视频', mov: 'MOV 视频',
  // 音频
  mp3: 'MP3 音频', flac: 'FLAC 音频', wav: 'WAV 音频', aac: 'AAC 音频',
  // 文档
  pdf: 'PDF 文档', txt: '文本文件', md: 'Markdown 文件',
  // 压缩包
  zip: 'ZIP 压缩包', rar: 'RAR 压缩包', '7z': '7z 压缩包', tar: 'TAR 归档',
  // 代码
  json: 'JSON 文件', js: 'JavaScript 文件', ts: 'TypeScript 文件',
  vue: 'Vue 文件', html: 'HTML 文件', css: 'CSS 文件',
  xml: 'XML 文件', sh: 'Shell 脚本', py: 'Python 脚本',
  // APK
  apk: 'Android 安装包',
};

function getFileType(file) {
  if (file.is_dir) return '文件夹';
  const ext = file.name.split('.').pop()?.toLowerCase() || '';
  return TYPE_MAP[ext] || (ext ? ext.toUpperCase() + ' 文件' : '文件');
}
</script>

<style scoped>
.file-list-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* 表头 */
.list-header {
  display: grid;
  grid-template-columns: 1fr 180px 140px 100px;
  padding: 0 8px;
  border-bottom: 1px solid var(--color-divider);
  background: var(--table-header-bg);
  flex-shrink: 0;
  position: sticky;
  top: 0;
  z-index: 2;

  > div {
    padding: 8px 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-secondary);
    user-select: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: color 0.15s;

    &:hover { color: var(--color-text-primary); }
  }
}

.sort-arrow {
  font-size: 10px;
  color: var(--color-primary);
}

/* 列表主体 */
.list-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* 文件行 */
.list-row {
  display: grid;
  grid-template-columns: 1fr 180px 140px 100px;
  padding: 0 8px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
  border-bottom: 1px solid transparent;

  &:hover {
    background: var(--table-row-hover);
    border-bottom-color: rgba(var(--color-primary-rgb), 0.12);
  }

  &.selected {
    background: rgba(var(--color-primary-rgb), 0.12);
    border-bottom-color: rgba(var(--color-primary-rgb), 0.18);
  }

  > div {
    padding: 6px 12px;
    font-size: 12px;
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
    min-width: 0;
  }
}

/* 名称列 */
.col-name {
  gap: 8px;

  .file-name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }
}

.symlink-badge {
  font-size: 10px;
  background: rgba(var(--color-info-rgb), 0.12);
  color: var(--color-info);
  padding: 1px 5px;
  border-radius: 3px;
  flex-shrink: 0;
}

/* 日期列 */
.col-date { color: var(--color-text-secondary); font-size: 11px; }
/* 类型列 */
.col-type { color: var(--color-text-secondary); font-size: 11px; }
/* 大小列 */
.col-size { color: var(--color-text-secondary); font-size: 11px; text-align: right; justify-content: flex-end; }

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 200px;
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
