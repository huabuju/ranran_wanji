<template>
  <div class="rom-download">
    <RomDownloadToolbar
      :current-level="currentLevel"
      :selected-brand="selectedBrand"
      :selected-model="selectedModel"
      :source-key="sourceKey"
      :source-options="sourceOptions"
      :model-search-raw="searchRaw"
      :rom-search-raw="romSearchRaw"
      :total-model-count="totalModelCount"
      :total-rom-count="totalRomCount"
      :model-count="filteredModels.length"
      :model-rom-count="filteredModelRomCount"
      :view-mode="viewMode"
      @go-level="goToLevel"
      @update:model-search-raw="searchRaw = $event"
      @update:rom-search-raw="romSearchRaw = $event"
      @update:view-mode="viewMode = $event"
      @update:source-key="handleSourceChange"
    />

    <RomSourceNotice
      :visible="currentLevel === 'roms' || currentLevel === 'models'"
      :source-label="currentSourceMeta.label"
      :source-description="currentSourceMeta.description"
    />

    <div class="content-area">
      <div v-if="isLoading || isModelLoading" class="loading-view">
        <div class="loader"></div>
        <div class="loader-text">
          {{ isLoading ? '正在加载完整的设备更新数据库...' : '正在从 XiaomiROM 加载该机型的在线 ROM 列表...' }}
        </div>
      </div>

      <RomBrandGrid v-else-if="currentLevel === 'brands'" :brands="brands" @select-brand="selectBrand" />
      <RomModelGrid v-else-if="currentLevel === 'models'" :models="filteredModels" @select-model="selectModel" />
      <RomListPanel
        v-else-if="currentLevel === 'roms'"
        :roms="filteredRoms"
        :view-mode="viewMode"
        :get-android-cls="getAndroidCls"
        :get-flash-type-cls="getFlashTypeCls"
        :get-flash-type-label="getFlashTypeLabel"
        @copy-url="handleCopyUrl"
        @download="addToDownloader"
      />
    </div>

    <UrlChoiceDialog
      :show="urlChoice.visible"
      :title="urlChoice.title"
      :description="urlChoice.description"
      :urls="urlChoice.urls"
      :action-label="urlChoice.actionLabel"
      @select="handleUrlChoiceSelect"
      @close="handleUrlChoiceClose"
    />
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import UrlChoiceDialog from '@/components/common/UrlChoiceDialog.vue';
import {
  DEFAULT_ROM_DATA_SOURCE,
  ensureModelRoms,
  getRomDataSourceMeta,
  getRomDataSourceOptions,
  loadBrandsData,
  resolveRomUrlsForEntry,
} from '@/config/loadRomsData.js';
import { debounce } from '@/utils/debounce.js';
import { getAndroidMajorVersion } from '@/utils/romData.js';
import { RomBrandGrid, RomDownloadToolbar, RomListPanel, RomModelGrid, RomSourceNotice } from './components';

const router = useRouter();

const brands = ref([]);
const isLoading = ref(true);
const isModelLoading = ref(false);
const currentLevel = ref('brands');
const selectedBrand = ref(null);
const selectedModel = ref(null);
const sourceKey = ref(DEFAULT_ROM_DATA_SOURCE);
const sourceOptions = getRomDataSourceOptions();
const viewMode = ref('grid');
const searchRaw = ref('');
const searchQuery = ref('');
const romSearchRaw = ref('');
const romSearchQuery = ref('');
const urlChoice = reactive({
  visible: false,
  title: '选择下载地址',
  description: '',
  actionLabel: '选择',
  urls: [],
  resolve: null,
});

onMounted(async () => {
  await loadBrandsForSource(sourceKey.value);
});

const setModelSearch = debounce((value) => { searchQuery.value = value; }, 600);
const setRomSearch = debounce((value) => { romSearchQuery.value = value; }, 600);

watch(searchRaw, (value) => setModelSearch(value));
watch(romSearchRaw, (value) => setRomSearch(value));

const currentSourceMeta = computed(() => getRomDataSourceMeta(sourceKey.value));

const filteredModels = computed(() => {
  if (!selectedBrand.value) return [];
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return selectedBrand.value.models;

  return selectedBrand.value.models.filter((item) => {
    const text = [item.name, item.key].filter(Boolean).join(' ').toLowerCase();
    return text.includes(query);
  });
});

const filteredRoms = computed(() => {
  if (!selectedModel.value) return [];
  const query = romSearchQuery.value.trim().toLowerCase();
  if (!query) return selectedModel.value.roms;

  return selectedModel.value.roms.filter((item) => {
    const text = [
      item.version,
      item.name,
      item.region,
      item.regionLabel,
      item.branch,
      item.flashType,
      item.androidLabel,
    ].filter(Boolean).join(' ').toLowerCase();

    return text.includes(query);
  });
});

const totalModelCount = computed(() => brands.value.reduce((sum, brand) => sum + (brand.models?.length || 0), 0));
const totalRomCount = computed(() => (
  brands.value.reduce(
    (sum, brand) => sum + (brand.models || []).reduce((modelSum, model) => modelSum + (model.romCount || model.roms?.length || 0), 0),
    0,
  )
));
const filteredModelRomCount = computed(() => (
  filteredModels.value.reduce((sum, model) => sum + (model.romCount || model.roms?.length || 0), 0)
));

function selectBrand(brand) {
  selectedBrand.value = brand;
  searchRaw.value = '';
  searchQuery.value = '';
  currentLevel.value = 'models';
}

async function selectModel(model) {
  selectedModel.value = model;
  romSearchRaw.value = '';
  romSearchQuery.value = '';
  currentLevel.value = 'roms';
  isModelLoading.value = true;

  try {
    await ensureModelRoms(model, sourceKey.value);
  } catch (error) {
    ElMessage.error(`加载 ${currentSourceMeta.value.label} 在线 ROM 列表失败: ${error}`);
  } finally {
    isModelLoading.value = false;
  }
}

function goToLevel(level) {
  if (level === 'brands') {
    currentLevel.value = 'brands';
    selectedBrand.value = null;
    selectedModel.value = null;
  } else if (level === 'models') {
    currentLevel.value = 'models';
    selectedModel.value = null;
  }
}

function getAndroidCls(android) {
  const majorVersion = getAndroidMajorVersion(android);
  if (majorVersion >= 15) return 'tag-a15';
  if (majorVersion === 14) return 'tag-a14';
  if (majorVersion === 13) return 'tag-a13';
  return 'tag-legacy';
}

function getFlashTypeLabel(type) {
  if (type === 'fastboot') return '线刷包';
  if (type === 'firmware') return '底包固件';
  return '卡刷包';
}

function getFlashTypeCls(type) {
  if (type === 'fastboot') return 'tag-fastboot';
  if (type === 'firmware') return 'tag-firmware';
  return 'tag-card';
}

async function getRomUrls(rom) {
  const urls = await resolveRomUrlsForEntry(rom, sourceKey.value);
  return urls.map((item) => String(item || '').trim()).filter(Boolean);
}

function getRomPageFallbackUrls(rom) {
  return Array.from(new Set(
    [rom?.sourceUrl, rom?.pageUrl, rom?.url]
      .map((item) => String(item || '').trim())
      .filter(Boolean),
  ));
}

async function getRomUrlsForCopy(rom) {
  try {
    return {
      urls: await getRomUrls(rom),
      usedFallback: false,
    };
  } catch (error) {
    console.error(`复制 ${rom?.version || 'ROM'} 地址时解析下载链接失败:`, error);
    return {
      urls: getRomPageFallbackUrls(rom),
      usedFallback: true,
    };
  }
}

function selectUrlFromList({ title, description, actionLabel, urls }) {
  const availableUrls = urls.map((item) => String(item || '').trim()).filter(Boolean);
  if (availableUrls.length <= 1) {
    return Promise.resolve(availableUrls[0] || '');
  }

  urlChoice.title = title;
  urlChoice.description = description;
  urlChoice.actionLabel = actionLabel;
  urlChoice.urls = availableUrls;
  urlChoice.visible = true;

  return new Promise((resolve) => {
    urlChoice.resolve = resolve;
  });
}

function finishUrlChoice(url = '') {
  urlChoice.visible = false;
  urlChoice.urls = [];
  urlChoice.resolve?.(url);
  urlChoice.resolve = null;
}

function handleUrlChoiceSelect(url) {
  finishUrlChoice(url);
}

function handleUrlChoiceClose() {
  finishUrlChoice('');
}

function extractFileNameFromUrl(url) {
  try {
    const parts = new URL(url).pathname.split('/').filter(Boolean);
    return decodeURIComponent(parts[parts.length - 1] || '');
  } catch {
    return '';
  }
}

async function handleCopyUrl(rom) {
  const { urls: resolvedUrls, usedFallback } = await getRomUrlsForCopy(rom);
  if (resolvedUrls.length === 0) {
    ElMessage.error('未解析到可用的下载链接');
    return;
  }

  const url = await selectUrlFromList({
    title: `复制 ${rom.version || 'ROM'} 地址`,
    description: '这个版本存在多个可用下载地址，请选择要复制的一个。',
    actionLabel: '复制',
    urls: resolvedUrls,
  });

  if (!url) return;

  try {
    await navigator.clipboard.writeText(url);
    ElMessage[usedFallback ? 'warning' : 'success'](
      usedFallback ? '未解析出直链，已复制下载页地址' : '下载地址已复制到剪贴板',
    );
  } catch {
    ElMessage.error('复制失败，请重试');
  }
}

async function addToDownloader(rom) {
  let resolvedUrls = [];

  try {
    resolvedUrls = await getRomUrls(rom);
  } catch (error) {
    console.error(`加入下载器前解析 ${rom?.version || 'ROM'} 下载链接失败:`, error);
    ElMessage.error(`未能从 ${currentSourceMeta.value.label} 下载页解析出直链`);
    return;
  }

  if (resolvedUrls.length === 0) {
    ElMessage.error('未解析到可用的下载链接');
    return;
  }

  const url = await selectUrlFromList({
    title: `下载 ${rom.version || 'ROM'}`,
    description: '这个版本存在多个可用下载地址，请选择要发送到下载器的一个。',
    actionLabel: '下载',
    urls: resolvedUrls,
  });

  if (!url) return;

  sessionStorage.setItem('pending_download_url', url);
  sessionStorage.setItem('pending_download_filename', rom.filename || extractFileNameFromUrl(url) || '');
  sessionStorage.setItem('pending_download_referer', String(rom?.sourceUrl || '').trim());
  router.push('/downloader');
}

async function refresh() {
  await loadBrandsForSource(sourceKey.value);
}

defineExpose({ refresh });

let latestSourceLoadToken = 0;

function resetBrowseState() {
  brands.value = [];
  currentLevel.value = 'brands';
  selectedBrand.value = null;
  selectedModel.value = null;
  searchRaw.value = '';
  searchQuery.value = '';
  romSearchRaw.value = '';
  romSearchQuery.value = '';
  isModelLoading.value = false;
}

async function loadBrandsForSource(nextSourceKey) {
  const token = ++latestSourceLoadToken;
  const nextMeta = getRomDataSourceMeta(nextSourceKey);

  isLoading.value = true;
  resetBrowseState();

  try {
    const nextBrands = await loadBrandsData(nextSourceKey);
    if (token !== latestSourceLoadToken) return;
    brands.value = nextBrands;
  } catch (error) {
    if (token !== latestSourceLoadToken) return;
    console.error(`加载 ${nextMeta.label} 数据失败:`, error);
    ElMessage.error(`无法加载 ${nextMeta.label} 机型数据，请重试: ${error}`);
  } finally {
    if (token === latestSourceLoadToken) {
      isLoading.value = false;
    }
  }
}

function handleSourceChange(nextSourceKey) {
  if (!nextSourceKey || nextSourceKey === sourceKey.value) return;
  sourceKey.value = nextSourceKey;
  void loadBrandsForSource(nextSourceKey);
}
</script>

<style lang="scss" scoped>
.rom-download {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 12px;
  background-color: transparent;
}

.content-area {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0 24px;
  background: transparent;
}

.loading-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--color-text-secondary);
  gap: 16px;
  padding: 0 24px;
}

.loader {
  border: 3px solid var(--surface-soft);
  border-top: 3px solid var(--color-primary);
  border-radius: 50%;
  width: 24px;
  height: 24px;
  animation: spin 1s linear infinite;
}

.loader-text { font-size: 14px; }
@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
</style>
