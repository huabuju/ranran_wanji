<template>
  <!-- 右键上下文菜单 -->
  <teleport to="body">
    <transition name="ctx-menu-fade">
      <div
        v-if="visible"
        class="ctx-menu"
        :style="menuStyle"
        @click.stop
        ref="menuRef"
      >
        <!-- 有选中文件时的操作 -->
        <template v-if="targetFile">
          <!-- 下载到电脑 -->
          <div class="ctx-item" @click="emit('pull', targetFile)">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd"/></svg>
            <span>下载到电脑</span>
          </div>

          <div class="ctx-divider" />

          <!-- 重命名 -->
          <div class="ctx-item" @click="emit('rename', targetFile)">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"/></svg>
            <span>重命名</span>
          </div>

          <!-- 复制路径 -->
          <div class="ctx-item" @click="emit('copy-path', targetFile)">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z"/><path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z"/></svg>
            <span>复制路径</span>
          </div>

          <div class="ctx-divider" />

          <!-- 删除 -->
          <div class="ctx-item danger" @click="emit('delete', targetFile)">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd"/></svg>
            <span>删除</span>
          </div>
        </template>

        <!-- 空白处右键（当前目录操作） -->
        <template v-else>
          <!-- 从电脑上传 -->
          <div class="ctx-item" @click="emit('push')">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6.293 6.707a1 1 0 010-1.414l3-3a1 1 0 011.414 0l3 3a1 1 0 01-1.414 1.414L11 5.414V13a1 1 0 11-2 0V5.414L7.707 6.707a1 1 0 01-1.414 0z" clip-rule="evenodd"/></svg>
            <span>上传文件到此处</span>
          </div>

          <!-- 上传文件夹 -->
          <div class="ctx-item" @click="emit('push-dir')">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/></svg>
            <span>上传文件夹到此处</span>
          </div>

          <div class="ctx-divider" />

          <!-- 新建文件夹 -->
          <div class="ctx-item" @click="emit('mkdir')">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path fill-rule="evenodd" d="M6 2a2 2 0 00-2 2v12a2 2 0 002 2h8a2 2 0 002-2V7.414A2 2 0 0015.414 6L12 2.586A2 2 0 0010.586 2H6zm5 6a1 1 0 10-2 0v2H7a1 1 0 100 2h2v2a1 1 0 102 0v-2h2a1 1 0 100-2h-2V8z" clip-rule="evenodd"/></svg>
            <span>新建文件夹</span>
          </div>

          <div class="ctx-divider" />

          <!-- 刷新 -->
          <div class="ctx-item" @click="emit('refresh')">
            <svg viewBox="0 0 20 20" fill="currentColor" width="14" height="14"><path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1z" clip-rule="evenodd"/></svg>
            <span>刷新</span>
          </div>
        </template>
      </div>
    </transition>
  </teleport>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
  x: { type: Number, default: 0 },
  y: { type: Number, default: 0 },
  targetFile: { type: Object, default: null },
});
const emit = defineEmits([
  'close',
  'pull', 'push', 'push-dir',
  'rename', 'delete', 'copy-path',
  'mkdir', 'refresh',
]);

const menuRef = ref(null);

const menuStyle = computed(() => ({
  left: props.x + 'px',
  top: props.y + 'px',
}));

function onClickOutside(e) {
  if (menuRef.value && !menuRef.value.contains(e.target)) {
    emit('close');
  }
}

onMounted(() => document.addEventListener('click', onClickOutside, true));
onBeforeUnmount(() => document.removeEventListener('click', onClickOutside, true));
</script>

<style scoped>
.ctx-menu {
  position: fixed;
  z-index: 9999;
  background: var(--surface-elevated);
  backdrop-filter: blur(18px);
  border: 1px solid var(--border-strong);
  border-radius: 8px;
  box-shadow: var(--shadow-card-hover);
  padding: 4px;
  min-width: 180px;
  user-select: none;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 12px;
  border-radius: 5px;
  font-size: 12.5px;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: background 0.12s, color 0.12s;

  &:hover {
    background: rgba(var(--color-primary-rgb), 0.1);
    color: var(--color-primary);

    svg { color: var(--color-primary); }
  }

  &.danger {
    color: var(--color-danger);
    svg { color: var(--color-danger); }
    &:hover { background: rgba(var(--color-danger-rgb), 0.12); color: var(--color-danger); }
  }

  svg { flex-shrink: 0; color: var(--color-text-secondary); }
}

.ctx-divider {
  height: 1px;
  background: var(--color-divider);
  margin: 3px 4px;
}

/* 过渡动画 */
.ctx-menu-fade-enter-active {
  animation: ctxFadeIn 0.12s ease-out;
}
.ctx-menu-fade-leave-active {
  animation: ctxFadeIn 0.08s ease-in reverse;
}
@keyframes ctxFadeIn {
  from { opacity: 0; transform: scale(0.95) translateY(-4px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}
</style>
