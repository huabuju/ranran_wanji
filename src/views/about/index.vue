<template>
  <div class="about-container">
    <AboutHeroSection
      title="然然玩机工具箱"
      :version="version"
      :build-time="buildTime"
      :runtime-path="runtimePathDisplay"
      :runtime-path-full="toolRuntimePath"
      description="专业的安卓设备管理工具"
    />

    <AboutInfoGrid :cards="infoCards" @card-click="handleCardClick" />

    <AboutTechTags :tags="tags" />

    <AboutActionSection
      :loading-changelog="loadingChangelog"
      @check-updates="checkUpdates"
      @show-changelog="handleShowChangelog"
    />

    <div class="about-footer section-fade-in">
      <p class="copyright">Copyright © 2025-{{ currentYear }} Android Toolkit. All rights reserved.</p>
    </div>

    <SponsorDialog ref="sponsorDialogRef" />

    <AboutChangelogDialog
      v-model:visible="showChangelogDialog"
      :changelog="updateInfo.changelog || []"
    />
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import SponsorDialog from '@/components/about/SponsorDialog.vue';
import { formatDateTime } from '@/utils/date';
import { checkUpdate, fetchUpdateInfo } from '@/utils/updater';
import { useUpdateStore } from '@/utils/updateStore';
import {
  AboutActionSection,
  AboutChangelogDialog,
  AboutHeroSection,
  AboutInfoGrid,
  AboutTechTags,
} from './components';

const { updateInfo } = useUpdateStore();
const sponsorDialogRef = ref(null);
const showChangelogDialog = ref(false);
const loadingChangelog = ref(false);

const version = ref('1.0.0');
const buildTime = ref('');
const currentYear = new Date().getFullYear();
const toolRuntimePath = ref('--');

const runtimePathDisplay = computed(() => {
  const path = String(toolRuntimePath.value || '').trim();
  if (!path || path === '--') {
    return '--';
  }

  if (path.length <= 64) {
    return path;
  }

  return `${path.slice(0, 28)}...${path.slice(-28)}`;
});

const tags = ['Tauri v2', 'Vue 3', 'Vite', 'javascript'];

const infoCards = [
  {
    icon: 'device',
    label: 'Tauri 框架',
    value: 'tauri',
    color: 'var(--color-info)',
    link: 'https://tauri.app/zh-cn/start/'
  },
  {
    icon: 'package',
    label: '官方频道',
    value: 'QQ群: 731971089',
    color: 'var(--color-success)',
    link: 'https://qm.qq.com/cgi-bin/qm/qr?k=I-WgATuEcB64VpQsx-N67HtGSw-7WeHF&jump_from=webapi&authKey=y2A+N7ofKxRdg2tFIchzGdbdmlPnTK6ap9DGzPucndqHQom7OmLL6ijZhU06ub5q'
  },
  {
    icon: 'github',
    label: '开源地址',
    value: '查看代码',
    color: 'var(--color-text-primary)',
    link: 'https://gitee.com/xiaowan12/toolkit-tauri-app'
  },
  {
    icon: 'power',
    label: '赞助支持',
    value: '支持作者',
    color: 'var(--color-danger)'
  }
];

async function handleCardClick(card) {
  if (card.label === '赞助支持') {
    sponsorDialogRef.value?.open();
    return;
  }

  if (card.link) {
    try {
      await openUrl(card.link);
    } catch (error) {
      console.error('Failed to open link:', error);
    }
  }
}

async function checkUpdates() {
  try {
    const result = await checkUpdate();

    if (result === 'no_update') {
      ElMessage({
        message: '当前已是最新版本',
        type: 'success',
      });
    } else if (result === 'failed') {
      ElMessage.error('检测更新失败，请检查网络连接');
    }
  } catch (error) {
    console.error('Manual update check failed:', error);
    ElMessage.error('检测更新过程中发生错误');
  }
}

async function handleShowChangelog() {
  if (updateInfo.value.changelog && updateInfo.value.changelog.length > 0) {
    showChangelogDialog.value = true;
    return;
  }

  loadingChangelog.value = true;
  try {
    await fetchUpdateInfo();
    showChangelogDialog.value = true;
  } catch (error) {
    ElMessage.error('获取更新日志失败');
  } finally {
    loadingChangelog.value = false;
  }
}

const refresh = async () => {
  try {
    version.value = await getVersion();
  } catch (error) {}
};

function resolveBuildTime() {
  if (typeof __APP_BUILD_TIME__ !== 'string' || !__APP_BUILD_TIME__) {
    return '';
  }

  return formatDateTime(__APP_BUILD_TIME__, 'YYYY-MM-DD HH:mm:ss');
}

defineExpose({ refresh });

onMounted(async () => {
  console.log('About page mounted');
  buildTime.value = resolveBuildTime();

  try {
    version.value = await getVersion();
  } catch (error) {
    console.error('Failed to get version:', error);
  }

  try {
    toolRuntimePath.value = await invoke('get_tool_runtime_path');
  } catch (error) {
    console.error('Failed to get tool runtime path:', error);
  }
});
</script>

<style lang="scss" scoped>
.about-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 36px 24px;
  background: transparent;
  overflow-y: auto;
}

.section-fade-in {
  animation: fadeInUp 0.6s ease-out forwards;
}

.about-footer .copyright {
  font-size: 12px;
  color: var(--color-text-muted);
  opacity: 0.7;
  margin: 0;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
