<template>
  <header
    class="top-bar"
    :class="{ 'is-focused': isWindowFocused }"
    @mousedown.left="handleTitlebarMouseDown"
    @dblclick="handleTitlebarDoubleClick"
  >
    <div class="top-bar-right no-window-drag">
      <button class="icon-button no-window-drag" type="button" aria-label="使用说明" @click="emit('usage-guide')">
        <span class="icon-slot" aria-hidden="true">
          <SmartIcon
            name="notification"
            color="currentColor"
            :size="13"
            :show-background="false"
            :show-decoration="false"
          />
        </span>
      </button>

      <div ref="menuAnchorRef" class="menu-anchor no-window-drag">
        <button
          class="icon-button no-window-drag"
          :class="{ 'is-active': showMenu }"
          type="button"
          aria-label="菜单"
          @click.stop="toggleMenu"
        >
          <span class="icon-slot" aria-hidden="true">
            <span class="menu-lines">
              <span></span>
              <span></span>
              <span></span>
            </span>
          </span>
        </button>

        <TopBarMenuPanel
          :show="showMenu"
          :items="menuItems"
          @select="handleMenuSelect"
        />
      </div>

      <div class="window-actions no-window-drag">
        <button class="window-button is-minimize no-window-drag" type="button" aria-label="最小化" @click="handleWindowMinimize">
          <span class="icon-slot" aria-hidden="true">
            <span class="window-glyph window-glyph--minimize"></span>
          </span>
        </button>
        <button class="window-button is-close no-window-drag" type="button" aria-label="关闭" @click="handleWindowClose">
          <span class="icon-slot" aria-hidden="true">
            <span class="window-glyph window-glyph--close"></span>
          </span>
        </button>
      </div>
    </div>
  </header>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import SmartIcon from '@/components/common/SmartIcon.vue';
import TopBarMenuPanel from '@/components/layout/TopBarMenuPanel.vue';
import { restartApplication } from '@/api/device';

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
  showMenu.value = !showMenu.value;
}

async function handleWindowMinimize() {
  await appWindow.minimize();
}

async function handleWindowClose() {
  await appWindow.close();
}

async function startWindowDragging() {
  await appWindow.startDragging();
}

async function toggleWindowMaximize() {
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
  --topbar-height: 44px;
  position: relative;
  height: var(--topbar-height);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 20px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.92), rgba(255, 255, 255, 0.82)),
    radial-gradient(circle at top left, rgba(96, 165, 250, 0.08), transparent 32%);
  border-bottom: 1px solid rgba(226, 232, 240, 0.86);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
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
  transition: background-color 0.14s ease, color 0.14s ease;
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.icon-button:hover,
.icon-button.is-active,
.window-button:hover {
  background: rgba(15, 23, 42, 0.05);
  color: var(--color-text-primary);
}

.icon-slot {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.icon-slot :deep(.smart-icon-container) {
  width: 16px !important;
  height: 16px !important;
  min-width: 16px;
  min-height: 16px;
  color: currentColor !important;
}

.icon-slot :deep(.smart-icon-svg) {
  width: 16px !important;
  height: 16px !important;
}

.menu-lines {
  width: 16px;
  height: 16px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 3px;

  span {
    display: block;
    width: 100%;
    height: 1.5px;
    border-radius: 999px;
    background: currentColor;
  }
}

.window-actions {
  display: flex;
  align-items: center;
  gap: 0;
  margin-left: 2px;
}

.window-glyph {
  position: relative;
  display: block;
  width: 16px;
  height: 16px;
  color: currentColor;
  flex-shrink: 0;
}

.window-glyph--minimize::before {
  content: '';
  position: absolute;
  left: 2px;
  right: 2px;
  bottom: 4px;
  height: 1.5px;
  background: currentColor;
}

.window-glyph--close::before,
.window-glyph--close::after {
  content: '';
  position: absolute;
  left: 7px;
  top: 1px;
  width: 1.4px;
  height: 14px;
  background: currentColor;
  border-radius: 999px;
}

.window-glyph--close::before {
  transform: rotate(45deg);
}

.window-glyph--close::after {
  transform: rotate(-45deg);
}

.top-bar.is-focused {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.55);
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
