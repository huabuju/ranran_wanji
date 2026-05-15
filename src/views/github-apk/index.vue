<template>
  <div class="github-apk-page">
    <GitHubApkToolbar
      v-model:repo="repoInput"
      v-model:github-token="githubToken"
      v-model:search-query="searchQuery"
      :repo-options="repoOptions"
      :loading="loading"
      :total="filteredAssets.length"
      @load="loadAssets"
      @refresh="refresh"
    />

    <GitHubApkFilterBar
      :filters="channelFilters"
      :current-filter="channelFilter"
      :repo="resolvedRepo"
      :release-count="releaseCount"
      :visible-count="filteredAssets.length"
      @change-filter="channelFilter = $event"
    />

    <div class="main-split-container">
      <GitHubApkTablePanel
        :columns="columns"
        :data="filteredAssets"
        :loading="loading"
        :format-size="formatSize"
        :format-count="formatCount"
        :format-date="formatDate"
        @download="downloadAsset"
        @copy-url="copyAssetUrl"
      />

      <FloatingLog :logs="logs" @clear="logs = []" />
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { fetchGithubApkAssets } from '@/api/githubApk';
import { getSystemDownloadDir } from '@/utils/systemPaths';
import { GitHubApkFilterBar, GitHubApkTablePanel, GitHubApkToolbar } from './components';

const DEFAULT_REPO = 'topjohnwu/Magisk';
const BUILTIN_REPOS = [DEFAULT_REPO, 'vvb2060/Magisk','tiann/KernelSU','bmax121/APatch','SukiSU-Ultra/SukiSU-Ultra','KernelSU-Next/KernelSU-Next','LyraVoid/FolkPatch'];
const RECENT_REPO_STORAGE_KEY = 'github_apk_recent_repos';
const GITHUB_TOKEN_STORAGE_KEY = 'github_apk_access_token';
const channelFilters = [
  { label: '全部', value: 'all' },
  { label: '正式版', value: 'stable' },
  { label: '预发布', value: 'prerelease' },
  { label: '草稿', value: 'draft' },
];

const repoInput = ref(DEFAULT_REPO);
const resolvedRepo = ref(DEFAULT_REPO);
const repoOptions = ref([...BUILTIN_REPOS]);
const githubToken = ref('');
const loading = ref(false);
const searchQuery = ref('');
const channelFilter = ref('all');
const assets = ref([]);
const releaseCount = ref(0);
const logs = ref([]);
const defaultSaveDir = ref('');

const filteredAssets = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();

  return assets.value.filter((item) => {
    if (channelFilter.value === 'stable' && (item.isPrerelease || item.isDraft)) {
      return false;
    }
    if (channelFilter.value === 'prerelease' && !item.isPrerelease) {
      return false;
    }
    if (channelFilter.value === 'draft' && !item.isDraft) {
      return false;
    }
    if (!query) {
      return true;
    }

    return [
      item.releaseTag,
      item.releaseName,
      item.assetName,
      item.channelLabel,
    ].some((value) => String(value || '').toLowerCase().includes(query));
  });
});

const columns = computed(() => [
  { key: 'release', title: '发行版', dataKey: 'release', width: 220, flexGrow: 1, flexShrink: 1 },
  { key: 'asset', title: 'APK 文件', dataKey: 'asset', width: 320, flexGrow: 1, flexShrink: 1 },
  { key: 'channel', title: '通道', dataKey: 'channel', width: 96, align: 'center' },
  { key: 'size', title: '大小', dataKey: 'size', width: 110, align: 'center' },
  { key: 'downloads', title: '下载量', dataKey: 'downloads', width: 110, align: 'center' },
  { key: 'publishedAt', title: '发布时间', dataKey: 'publishedAt', width: 170, align: 'center' },
  { key: 'actions', title: '操作', width: 170, align: 'center', fixed: 'right' },
]);

function addLog(content, type = 'info') {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
  logs.value.push({ time, content, type });
}

function normalizeRepoValue(value) {
  return String(value || '').trim();
}

function setRepoOptions(nextOptions) {
  const deduped = Array.from(new Set(
    nextOptions
      .map(normalizeRepoValue)
      .filter(Boolean),
  )).slice(0, 8);

  repoOptions.value = deduped.length ? deduped : [...BUILTIN_REPOS];
}

function loadRecentRepoOptions() {
  try {
    const raw = localStorage.getItem(RECENT_REPO_STORAGE_KEY);
    const parsed = JSON.parse(raw || '[]');
    if (Array.isArray(parsed)) {
      setRepoOptions([...BUILTIN_REPOS, ...parsed]);
      return;
    }
  } catch {}

  setRepoOptions(BUILTIN_REPOS);
}

function saveRecentRepo(repo) {
  const normalized = normalizeRepoValue(repo);
  if (!normalized) {
    return;
  }

  setRepoOptions([normalized, ...repoOptions.value]);

  try {
    localStorage.setItem(RECENT_REPO_STORAGE_KEY, JSON.stringify(repoOptions.value));
  } catch {}
}

function loadGithubToken() {
  try {
    githubToken.value = String(localStorage.getItem(GITHUB_TOKEN_STORAGE_KEY) || '').trim();
  } catch {
    githubToken.value = '';
  }
}

function saveGithubToken(token) {
  try {
    const normalized = String(token || '').trim();
    if (normalized) {
      localStorage.setItem(GITHUB_TOKEN_STORAGE_KEY, normalized);
    } else {
      localStorage.removeItem(GITHUB_TOKEN_STORAGE_KEY);
    }
  } catch {}
}

function decorateAsset(item) {
  let channelLabel = '正式版';
  let channelClass = 'is-stable';

  if (item.isDraft) {
    channelLabel = '草稿';
    channelClass = 'is-draft';
  } else if (item.isPrerelease) {
    channelLabel = '预发布';
    channelClass = 'is-prerelease';
  }

  return {
    ...item,
    channelLabel,
    channelClass,
  };
}

async function loadAssets() {
  const repo = normalizeRepoValue(repoInput.value);
  const token = String(githubToken.value || '').trim();
  if (!repo) {
    ElMessage.warning('请输入 GitHub 仓库');
    return;
  }

  loading.value = true;
  addLog(`正在读取 ${repo} 的 GitHub 发行版 APK 列表...`);

  try {
    const response = await fetchGithubApkAssets(repo, token);
    resolvedRepo.value = response.repo || repo;
    repoInput.value = resolvedRepo.value;
    saveRecentRepo(resolvedRepo.value);
    saveGithubToken(token);
    releaseCount.value = Number(response.releaseCount || 0);
    assets.value = (response.assets || []).map(decorateAsset);
    addLog(`读取完成：共发现 ${assets.value.length} 个 APK，来自 ${releaseCount.value} 个发行版`, 'success');
    ElMessage.success(`已加载 ${assets.value.length} 个 APK`);
  } catch (error) {
    assets.value = [];
    releaseCount.value = 0;
    addLog(`读取失败: ${error}`, 'error');
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function copyAssetUrl(row) {
  try {
    await navigator.clipboard.writeText(row.downloadUrl);
    addLog(`已复制下载地址: ${row.assetName}`, 'success');
    ElMessage.success('下载地址已复制');
  } catch (error) {
    addLog(`复制地址失败: ${error}`, 'error');
    ElMessage.error('复制失败，请重试');
  }
}

async function downloadAsset(row) {
  if (!defaultSaveDir.value) {
    defaultSaveDir.value = await getSystemDownloadDir();
  }

  const id = `dl_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
  try {
    addLog(`已提交下载任务: ${row.assetName}`);
    await invoke('start_download', {
      id,
      url: row.downloadUrl,
      saveDir: defaultSaveDir.value,
      fileName: row.assetName,
      threads: 8,
      referer: row.releasePageUrl || null,
    });
    ElMessage.success(`已加入下载器: ${row.assetName}`);
  } catch (error) {
    addLog(`提交下载任务失败: ${error}`, 'error');
    ElMessage.error(String(error));
  }
}

function formatSize(value) {
  const size = Number(value || 0);
  if (!Number.isFinite(size) || size <= 0) {
    return '-';
  }

  const units = ['B', 'KB', 'MB', 'GB'];
  let next = size;
  let unitIndex = 0;
  while (next >= 1024 && unitIndex < units.length - 1) {
    next /= 1024;
    unitIndex += 1;
  }

  return `${next >= 100 ? next.toFixed(0) : next.toFixed(1)} ${units[unitIndex]}`;
}

function formatCount(value) {
  const count = Number(value || 0);
  if (!Number.isFinite(count)) {
    return '-';
  }

  return count.toLocaleString('zh-CN');
}

function formatDate(value) {
  if (!value) {
    return '-';
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString('zh-CN', {
    hour12: false,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

async function refresh() {
  await loadAssets();
}

defineExpose({ refresh });

onMounted(async () => {
  loadRecentRepoOptions();
  loadGithubToken();
  defaultSaveDir.value = await getSystemDownloadDir();
  await loadAssets();
});

watch(githubToken, (value) => {
  saveGithubToken(value);
});
</script>

<style lang="scss" scoped>
.github-apk-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 20px;
  background-color: transparent;
}

.main-split-container {
  flex: 1;
  display: flex;
  overflow: hidden;
  gap: 20px;
  background-color: transparent;
}
</style>
