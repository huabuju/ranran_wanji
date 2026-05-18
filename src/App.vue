<template>
  <RuntimeBootstrapScreen
    v-if="!runtimeReady"
    :message="runtimeMessage"
    :progress="runtimeProgress"
    :error="runtimeError"
    :preparing="runtimePreparing"
    :phase="runtimePhase"
    @retry="ensureRuntimeReady"
  />

  <div v-else class="app-layout">
    <Sidebar />

    <div class="main-area">
      <TopBar
        :is-closing="isAppClosing"
        @refresh="handleRefresh"
        @open-settings="openSettingsDialog"
        @usage-guide="openUsageGuide"
      />

      <main class="page-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" ref="activeComponentRef" />
          </transition>
        </router-view>
      </main>

      <StatusBar />
    </div>
    <UpdateDialog />
    <CheckUpdateLoading />
    <GlassLoading
      :show="isAppClosing"
      title="正在退出工具..."
      description="正在清理运行中工具进程，请稍候。"
    />
    <OverviewUsageGuideDialog ref="usageGuideDialogRef" />
    <SettingsDialog ref="settingsDialogRef" />
  </div>
</template>

<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useRoute } from 'vue-router';
import GlassLoading from '@/components/common/GlassLoading.vue';
import RuntimeBootstrapScreen from '@/components/common/RuntimeBootstrapScreen.vue';
import UpdateDialog from '@/components/common/UpdateDialog.vue';
import CheckUpdateLoading from '@/components/common/CheckUpdateLoading.vue';
import OverviewUsageGuideDialog from '@/components/common/OverviewUsageGuideDialog.vue';
import Sidebar from '@/components/layout/Sidebar.vue';
import TopBar from '@/components/layout/TopBar.vue';
import StatusBar from '@/components/layout/StatusBar.vue';
import SettingsDialog from '@/components/settings/SettingsDialog.vue';
import { initGlobalAutoMirror } from '@/utils/scrcpyStore';
import { useDeviceStore } from '@/utils/deviceStore';
import { checkUpdate } from '@/utils/updater';

const activeComponentRef = ref(null);
const settingsDialogRef = ref(null);
const usageGuideDialogRef = ref(null);
const runtimeReady = ref(false);
const runtimePreparing = ref(false);
const runtimeMessage = ref('正在检查运行时依赖...');
const runtimeProgress = ref(0);
const runtimeError = ref('');
const runtimePhase = ref('check');
const isAppClosing = ref(false);
const route = useRoute();
const { selectedSerial } = useDeviceStore();
const OVERVIEW_USAGE_GUIDE_SHOWN_KEY = 'overview_usage_guide_shown';
let unlistenRuntimeProgress = null;
let unlistenAppClosing = null;
const runtimeSourceLabels = {
  dev: '开发环境本地 bin',
  bundle: '安装包内置资源',
  'runtime-cache': '本地缓存',
  'runtime-cache-offline': '本地缓存（离线模式）',
  download: '在线下载',
};

watch(selectedSerial, (newVal, oldVal) => {
  if (runtimeReady.value && newVal && oldVal && newVal !== oldVal) {
    handleRefresh();
  }
});

watch(
  [runtimeReady, () => route.name],
  ([ready, routeName]) => {
    if (ready && routeName === 'DeviceOverview') {
      void tryOpenUsageGuideOnFirstOverviewEntry();
    }
  },
  { immediate: true }
);

onMounted(async () => {
  unlistenRuntimeProgress = await listen('runtime-assets-progress', (event) => {
    const payload = event.payload || {};
    runtimePhase.value = payload.phase || 'check';
    runtimeMessage.value = payload.message || '正在准备运行时依赖...';
    runtimeProgress.value = Number(payload.progress || 0);
  });

  unlistenAppClosing = await listen('app-closing', () => {
    isAppClosing.value = true;
  });

  await ensureRuntimeReady();
});

onBeforeUnmount(() => {
  if (typeof unlistenRuntimeProgress === 'function') {
    unlistenRuntimeProgress();
    unlistenRuntimeProgress = null;
  }

  if (typeof unlistenAppClosing === 'function') {
    unlistenAppClosing();
    unlistenAppClosing = null;
  }
});

async function ensureRuntimeReady() {
  runtimePreparing.value = true;
  runtimeReady.value = false;
  runtimeError.value = '';

  try {
    const result = await invoke('prepare_runtime_assets');
    const sourceLabel = runtimeSourceLabels[result.source] || result.source || '未知来源';
    runtimeMessage.value = `运行时依赖已就绪，来源：${sourceLabel}`;
    runtimeProgress.value = 100;
    runtimePhase.value = 'ready';

    try {
      await invoke('warmup_platform_tools');
    } catch (error) {
      console.warn('Failed to warm up platform tools:', error);
    }

    initGlobalAutoMirror();

    await new Promise((resolve) => {
      setTimeout(resolve, 400);
    });

    runtimeReady.value = true;
    void checkUpdate({ silent: true });
  } catch (error) {
    runtimeError.value = error?.toString?.() || '运行时依赖初始化失败';
    runtimeMessage.value = '运行时依赖准备失败，请确认安装包中的 bin 资源完整后重试。';
  } finally {
    runtimePreparing.value = false;
  }
}

async function handleRefresh() {
  await nextTick();

  if (!activeComponentRef.value) {
    ElMessage.warning('当前页面还在加载中，请稍后再试。');
    return;
  }

  if (typeof activeComponentRef.value.refresh !== 'function') {
    ElMessage.info('当前页面暂不支持刷新');
    return;
  }

  try {
    await activeComponentRef.value.refresh();
  } catch (error) {
    console.error('Failed to refresh current page:', error);
    ElMessage.error(`刷新失败：${error}`);
  }
}

function openUsageGuide() {
  if (activeComponentRef.value && typeof activeComponentRef.value.openUsageGuide === 'function') {
    activeComponentRef.value.openUsageGuide();
    return;
  }

  if (usageGuideDialogRef.value && typeof usageGuideDialogRef.value.open === 'function') {
    usageGuideDialogRef.value.open();
  }
}

function hasShownOverviewUsageGuide() {
  return localStorage.getItem(OVERVIEW_USAGE_GUIDE_SHOWN_KEY) === 'true';
}

function markOverviewUsageGuideAsShown() {
  localStorage.setItem(OVERVIEW_USAGE_GUIDE_SHOWN_KEY, 'true');
}

async function tryOpenUsageGuideOnFirstOverviewEntry() {
  if (hasShownOverviewUsageGuide()) {
    return;
  }

  await nextTick();

  if (!usageGuideDialogRef.value || typeof usageGuideDialogRef.value.open !== 'function') {
    return;
  }

  markOverviewUsageGuideAsShown();
  usageGuideDialogRef.value.open();
}

function openSettingsDialog() {
  if (settingsDialogRef.value && typeof settingsDialogRef.value.open === 'function') {
    settingsDialogRef.value.open();
  }
}
</script>

<style>
#app {
  width: 100%;
  height: 100%;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

<style lang="scss" scoped>
.app-layout {
  display: flex;
  width: 100%;
  height: 100%;
  background: var(--bg-app);
  overflow: hidden;

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;

    .page-content {
      flex: 1;
      padding: 16px;
      overflow: auto;
      display: flex;
      flex-direction: column;

      .page-placeholder {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 12px;
        color: var(--color-text-muted);

        p {
          font-size: 14px;
        }
      }

      .placeholder-icon {
        font-size: 40px;
      }
    }
  }
}
</style>
