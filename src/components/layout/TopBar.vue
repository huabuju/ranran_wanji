<template>
  <header
    class="top-bar"
    :class="{ 'is-focused': isWindowFocused, 'is-busy': isClosing }"
    @mousedown.left="handleTitlebarMouseDown"
    @dblclick="handleTitlebarDoubleClick"
  >
    <div class="top-bar-right no-window-drag">
      <button
        class="icon-button no-window-drag"
        type="button"
        aria-label="使用说明"
        :disabled="isClosing"
        @click="emit('usage-guide')"
      >
        <span class="icon-slot" aria-hidden="true">
          <svg class="titlebar-icon" viewBox="0 0 16 16" focusable="false">
            <path d="M12.25 6.75a4.25 4.25 0 0 0-8.5 0c0 4.25-1.75 5.5-1.75 5.5h12s-1.75-1.25-1.75-5.5" />
            <path d="M9.25 14a1.5 1.5 0 0 1-2.5 0" />
          </svg>
        </span>
      </button>

      <div ref="menuAnchorRef" class="menu-anchor no-window-drag">
        <button
          class="icon-button no-window-drag"
          :class="{ 'is-active': showMenu }"
          type="button"
          aria-label="菜单"
          :disabled="isClosing"
          @click.stop="toggleMenu"
        >
          <span class="icon-slot" aria-hidden="true">
            <svg class="titlebar-icon" viewBox="0 0 16 16" focusable="false">
              <path d="M3 4.5h10" />
              <path d="M3 8h10" />
              <path d="M3 11.5h10" />
            </svg>
          </span>
        </button>

        <TopBarMenuPanel
          :show="showMenu"
          :items="menuItems"
          @select="handleMenuSelect"
        />
      </div>

      <div class="window-actions no-window-drag">
        <button
          class="window-button is-minimize no-window-drag"
          type="button"
          aria-label="最小化"
          :disabled="isClosing"
          @click="handleWindowMinimize"
        >
          <span class="icon-slot" aria-hidden="true">
            <svg class="titlebar-icon" viewBox="0 0 16 16" focusable="false">
              <path d="M3 11.5h10" />
            </svg>
          </span>
        </button>
        <button
          class="window-button is-close no-window-drag"
          type="button"
          aria-label="关闭"
          :disabled="isClosing"
          @click="handleWindowClose"
        >
          <span class="icon-slot" aria-hidden="true">
            <svg class="titlebar-icon" viewBox="0 0 16 16" focusable="false">
              <path d="M4 4l8 8" />
              <path d="M12 4l-8 8" />
            </svg>
          </span>
        </button>
      </div>
    </div>
  </header>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import TopBarMenuPanel from '@/components/layout/TopBarMenuPanel.vue';
import { restartApplication } from '@/api/device';

const props = defineProps({
  isClosing: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['refresh', 'open-settings', 'usage-guide']);
const appWindow = getCurrentWindow();

const showMenu = ref(false);
const isMaximized = ref(false);
const isWindowFocused = ref(true);
const menuAnchorRef = ref(null);

let unlistenFocusChanged = null;
let unlistenResized = null;
let unlistenDocumentClick = null;

const menuItems = computed(() => [
  { key: 'refresh-page', label: '刷新页面', icon: 'refresh' },
  { key: 'settings', label: '应用设置', icon: 'settings' },
  { key: 'restart-service', label: '重启服务', icon: 'reboot' },
]);

onMounted(async () => {
  await syncWindowState();

  unlistenFocusChanged = await appWindow.onFocusChanged(({ payload }) => {
    isWindowFocused.value = payload;
  });

  unlistenResized = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
  });

  const handleOutsideClick = (event) => {
    if (!menuAnchorRef.value?.contains(event.target)) {
      showMenu.value = false;
    }
  };

  document.addEventListener('click', handleOutsideClick);
  unlistenDocumentClick = () => {
    document.removeEventListener('click', handleOutsideClick);
  };
});

onBeforeUnmount(() => {
  if (typeof unlistenFocusChanged === 'function') {
    unlistenFocusChanged();
    unlistenFocusChanged = null;
  }

  if (typeof unlistenResized === 'function') {
    unlistenResized();
    unlistenResized = null;
  }

  if (typeof unlistenDocumentClick === 'function') {
    unlistenDocumentClick();
    unlistenDocumentClick = null;
  }
});

async function syncWindowState() {
  try {
    isMaximized.value = await appWindow.isMaximized();
    isWindowFocused.value = await appWindow.isFocused();
  } catch (error) {
    console.error('Failed to sync window state:', error);
  }
}

function toggleMenu() {
  if (props.isClosing) {
    return;
  }

  showMenu.value = !showMenu.value;
}

async function handleWindowMinimize() {
  if (props.isClosing) {
    return;
  }

  await appWindow.minimize();
}

async function handleWindowClose() {
  if (props.isClosing) {
    return;
  }

  showMenu.value = false;
  await appWindow.close();
}

async function startWindowDragging() {
  if (props.isClosing) {
    return;
  }

  await appWindow.startDragging();
}

async function toggleWindowMaximize() {
  if (props.isClosing) {
    return;
  }

  await appWindow.toggleMaximize();
  isMaximized.value = await appWindow.isMaximized();
}

function isDragExcludedTarget(target) {
  return target instanceof Element && Boolean(target.closest('.no-window-drag'));
}

function handleTitlebarMouseDown(event) {
  if (isDragExcludedTarget(event.target)) {
    return;
  }

  void startWindowDragging();
}

function handleTitlebarDoubleClick(event) {
  if (isDragExcludedTarget(event.target)) {
    return;
  }

  void toggleWindowMaximize();
}

async function handleMenuSelect(item) {
  if (props.isClosing) {
    return;
  }

  showMenu.value = false;

  if (item.key === 'refresh-page') {
    emit('refresh');
    return;
  }

  if (item.key === 'settings') {
    emit('open-settings');
    return;
  }

  if (item.key === 'restart-service') {
    await restartApplication();
  }
}
</script>

<style lang="scss" scoped>
.top-bar {
  padding-right: 8px;
  box-sizing: border-box;
  --topbar-height: 44px;
  position: relative;
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 20px;
  background:
    linear-gradient(180deg, var(--bg-header), var(--surface-elevated)),
    radial-gradient(circle at top left, rgba(var(--color-primary-rgb), 0.08), transparent 32%);
  border-bottom: 1px solid var(--border-soft);
  backdrop-filter: var(--blur-glass);
  -webkit-backdrop-filter: var(--blur-glass);
  flex-shrink: 0;
  z-index: 30;
  user-select: none;
}

.top-bar-right {
  display: flex;
  align-items: center;
}

.top-bar-left {
  flex: 1;
  min-width: 0;
  min-height: 100%;
}

.top-bar-right {
  gap: 2px;
  flex-shrink: 0;
}

.menu-anchor {
  position: relative;
}

.icon-button,
.window-button {
  width: 36px;
  height: 36px;
  padding: 0;
  border: none;
  border-radius: 10px;
  background: transparent;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.14s ease, color 0.14s ease, opacity 0.14s ease;
  flex-shrink: 0;
  color: color-mix(in srgb, var(--color-text-secondary) 78%, transparent);
}

.icon-button:hover,
.icon-button.is-active,
.window-button:hover {
  background: var(--bg-hover-subtle);
  color: var(--color-text-primary);
}

.icon-button:disabled,
.window-button:disabled {
  cursor: wait;
  opacity: 0.58;
}

.icon-slot {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.titlebar-icon {
  width: 16px;
  height: 16px;
  display: block;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.window-actions {
  display: flex;
  align-items: center;
  gap: 0;
  margin-left: 2px;
}

.top-bar.is-focused {
  box-shadow: inset 0 1px 0 var(--border-strong);
}

.top-bar.is-busy {
  pointer-events: none;
}

@media (max-width: 960px) {
  .top-bar {
    padding-inline: 14px;
  }
}

@media (max-width: 720px) {
  .top-bar {
    gap: 10px;
  }
}
</style>
