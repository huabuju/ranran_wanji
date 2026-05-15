<template>
  <div class="apps-management">
    <AppsToolbar v-model:search-query="searchQuery" :total="filteredPackages.length" @install="handleInstallApk" @refresh="fetchPackages" />

    <AppsFilterBar
      :app-type="appType"
      :status-filter="statusFilter"
      :selected-count="selectedRows.length"
      :is-batch-extracting="isBatchExtracting"
      :is-batch-uninstalling="isBatchUninstalling"
      :is-batch-network-blocking="isBatchNetworkBlocking"
      :is-batch-network-unblocking="isBatchNetworkUnblocking"
      :batch-extract-progress="batchExtractProgress"
      :batch-uninstall-progress="batchUninstallProgress"
      :batch-network-block-progress="batchNetworkBlockProgress"
      :batch-network-unblock-progress="batchNetworkUnblockProgress"
      @change-app-type="handleChangeAppType"
      @change-status-filter="handleChangeStatusFilter"
      @batch-extract="handleBatchExtractApk"
      @batch-uninstall="handleBatchUninstall"
      @batch-block-network="handleBatchBlockNetwork"
      @batch-unblock-network="handleBatchUnblockNetwork"
      @cancel-extract="cancelBatchExtract"
      @cancel-uninstall="cancelBatchUninstall"
      @cancel-block-network="cancelBatchNetworkBlock"
      @cancel-unblock-network="cancelBatchNetworkUnblock"
    />

    <div class="main-split-container">
      <AppsTablePanel
        :columns="columns"
        :data="filteredPackages"
        :loading="loading"
        :app-labels="appLabels"
        :pkg-details="pkgDetails"
        :row-event-handlers="rowEventHandlers"
        @copy-path="copyPath"
        @toggle-freeze="handleToggleFreeze"
        @row-action="handleRowAction"
      />

      <FloatingLog :logs="logs" @clear="logs = []" />
    </div>
  </div>
</template>

<script setup>
import { computed, h, onMounted, ref } from 'vue';
import { join } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';
import { ElCheckbox, ElMessage, ElMessageBox } from 'element-plus';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { AppsFilterBar, AppsTablePanel, AppsToolbar } from './components';
import {
  extractApk,
  fetchAppLabels as apiFetchAppLabels,
  fetchDisabledPackages,
  fetchPackageDetail,
  fetchPackages as apiFetchPackages,
  installPackage,
  managePackage,
} from '@/api/device';

const loading = ref(false);
const appType = ref(true);
const packages = ref([]);
const pkgDetails = ref({});
const appLabels = ref({});
const searchQuery = ref('');
const statusFilter = ref('all');
const selectedPackage = ref(null);
const logs = ref([]);
const selectedRows = ref([]);

const isBatchExtracting = ref(false);
const isBatchUninstalling = ref(false);
const isBatchNetworkBlocking = ref(false);
const isBatchNetworkUnblocking = ref(false);

const extractCancelled = ref(false);
const uninstallCancelled = ref(false);
const networkBlockCancelled = ref(false);
const networkUnblockCancelled = ref(false);

const batchExtractProgress = ref({ current: 0, total: 0 });
const batchUninstallProgress = ref({ current: 0, total: 0 });
const batchNetworkBlockProgress = ref({ current: 0, total: 0 });
const batchNetworkUnblockProgress = ref({ current: 0, total: 0 });

const cancelBatchExtract = () => { extractCancelled.value = true; };
const cancelBatchUninstall = () => { uninstallCancelled.value = true; };
const cancelBatchNetworkBlock = () => { networkBlockCancelled.value = true; };
const cancelBatchNetworkUnblock = () => { networkUnblockCancelled.value = true; };

const filteredPackages = computed(() => {
  let list = packages.value;
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    list = list.filter((pkg) => {
      const labelData = appLabels.value[pkg];
      const labelMatch = labelData?.label?.toLowerCase().includes(query);
      const uidMatch = labelData?.uid !== undefined && String(labelData.uid).includes(query);
      return pkg.toLowerCase().includes(query) || labelMatch || uidMatch;
    });
  }

  if (statusFilter.value === 'running') {
    list = list.filter((pkg) => pkgDetails.value[pkg]?.is_enabled !== false);
  } else if (statusFilter.value === 'frozen') {
    list = list.filter((pkg) => pkgDetails.value[pkg]?.is_enabled === false);
  }
  return list;
});

const isAllSelected = computed(() => filteredPackages.value.length > 0 && selectedRows.value.length === filteredPackages.value.length);
const isIndeterminate = computed(() => selectedRows.value.length > 0 && selectedRows.value.length < filteredPackages.value.length);

function toggleSelectAll(value) {
  selectedRows.value = value ? [...filteredPackages.value] : [];
}

function toggleSelection(pkg, value) {
  if (value) {
    if (!selectedRows.value.includes(pkg)) selectedRows.value.push(pkg);
  } else {
    selectedRows.value = selectedRows.value.filter((item) => item !== pkg);
  }
}

const rowEventHandlers = {
  onClick: ({ rowData }) => handleRowClick(rowData),
};

const columns = computed(() => [
  {
    key: 'selection',
    width: 50,
    align: 'center',
    headerCellRenderer: () => h(ElCheckbox, {
      modelValue: isAllSelected.value,
      indeterminate: isIndeterminate.value,
      'onUpdate:modelValue': toggleSelectAll,
    }),
    cellRenderer: ({ rowData }) => h(ElCheckbox, {
      modelValue: selectedRows.value.includes(rowData),
      'onUpdate:modelValue': (value) => toggleSelection(rowData, value),
      onClick: (event) => event.stopPropagation(),
    }),
  },
  { key: 'name', title: '应用名称', dataKey: 'name', width: 200, flexGrow: 1, flexShrink: 1 },
  { key: 'package', title: '包名', dataKey: 'package', width: 250, flexGrow: 1, flexShrink: 1 },
  { key: 'version', title: '版本', dataKey: 'version', width: 100, align: 'center' },
  { key: 'uid', title: 'UID', dataKey: 'uid', width: 80, align: 'center' },
  { key: 'path', title: '安装路径', dataKey: 'path', width: 300, flexGrow: 1, flexShrink: 1 },
  { key: 'actions', title: '操作', width: 220, align: 'center', fixed: 'right' },
]);

function addLog(content, type = 'info') {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
  logs.value.push({ time, content, type });
}

function sanitizeExportFileNamePart(value) {
  const sanitized = String(value || '')
    .trim()
    .replace(/[<>:"/\\|?*\u0000-\u001F]/g, '_')
    .replace(/\s+/g, ' ')
    .replace(/\.+$/g, '')
    .trim();

  return sanitized || 'unknown';
}

function buildApkExportBaseName(pkg, usedNames = new Set()) {
  const appLabel = appLabels.value[pkg]?.label || pkg;
  const versionName = appLabels.value[pkg]?.version_name;
  const safeLabel = sanitizeExportFileNamePart(appLabel);
  const safePackageName = sanitizeExportFileNamePart(pkg);
  const safeVersionName = versionName ? sanitizeExportFileNamePart(versionName) : '';

  let baseName = safeVersionName
    ? `${safeLabel}_${safePackageName}_v${safeVersionName}`
    : `${safeLabel}_${safePackageName}`;

  let dedupeIndex = 2;
  while (usedNames.has(baseName.toLowerCase())) {
    baseName = safeVersionName
      ? `${safeLabel}_${safePackageName}_v${safeVersionName}_${dedupeIndex}`
      : `${safeLabel}_${safePackageName}_${dedupeIndex}`;
    dedupeIndex += 1;
  }

  usedNames.add(baseName.toLowerCase());
  return baseName;
}

async function fetchPackages() {
  loading.value = true;
  packages.value = [];
  pkgDetails.value = {};
  appLabels.value = {};
  try {
    addLog(`正在获取${appType.value ? '用户' : '系统'}应用列表...`);
    const [list, disabledList] = await Promise.all([
      apiFetchPackages(appType.value),
      fetchDisabledPackages().catch(() => []),
    ]);
    packages.value = list;
    const disabledSet = new Set(disabledList);
    const detailMap = {};
    for (const pkg of list) {
      detailMap[pkg] = { ...(pkgDetails.value[pkg] || {}), is_enabled: !disabledSet.has(pkg) };
    }
    pkgDetails.value = detailMap;
    addLog(`成功获取 ${list.length} 个应用（其中 ${disabledList.length} 个已冻结）`, 'success');
    await fetchAppLabels();
  } catch (error) {
    addLog(`获取失败: ${error}`, 'error');
  } finally {
    loading.value = false;
  }
}

async function fetchAppLabels() {
  try {
    const labels = await apiFetchAppLabels(appType.value);
    const labelMap = {};
    labels.forEach((item) => {
      labelMap[item.package_name] = {
        label: item.label,
        uid: item.uid,
        version_name: item.version_name,
        source_dir: item.source_dir || '',
      };
    });
    appLabels.value = labelMap;
  } catch (error) {
    addLog(`获取应用名失败: ${error}`, 'error');
  }
}

async function handleRowClick(pkg) {
  try {
    const detail = await fetchPackageDetail(pkg);
    selectedPackage.value = detail;
    pkgDetails.value[pkg] = detail;
    addLog(`选中应用: ${pkg}`);
  } catch (error) {
    addLog(`获取详情失败: ${error}`, 'error');
  }
}

async function handleToggleFreeze(pkg, action) {
  const label = action === 'freeze' ? '冻结' : '解冻';
  try {
    addLog(`正在${label}应用: ${pkg}...`);
    const result = await managePackage(pkg, action);
    addLog(`${label}成功: ${result}`, 'success');
    const enabled = action !== 'freeze';
    pkgDetails.value[pkg] = { ...(pkgDetails.value[pkg] || {}), is_enabled: enabled };
    if (selectedPackage.value?.package_name === pkg) {
      selectedPackage.value = { ...selectedPackage.value, is_enabled: enabled };
    }
  } catch (error) {
    addLog(`${label}失败: ${error}`, 'error');
  }
}

async function handleRowAction(pkg, action) {
  const actionMap = { stop: '强行停止', clear: '清除数据' };
  try {
    addLog(`正在执行: ${actionMap[action]} -> ${pkg}...`);
    const result = await managePackage(pkg, action);
    addLog(`${actionMap[action]}成功: ${result}`, 'success');
  } catch (error) {
    addLog(`${actionMap[action]}失败: ${error}`, 'error');
  }
}

async function handleBatchExtractApk() {
  if (selectedRows.value.length === 0) return;
  try {
    await ElMessageBox.confirm(
      `即将批量提取 ${selectedRows.value.length} 个应用的 APK，请选择保存目录。`,
      '批量提取 APK',
      { confirmButtonText: '选择目录', cancelButtonText: '取消', type: 'info' }
    );
  } catch {
    return;
  }

  const dirPath = await open({ directory: true, multiple: false });
  if (!dirPath) return;

  isBatchExtracting.value = true;
  extractCancelled.value = false;
  batchExtractProgress.value = { current: 0, total: selectedRows.value.length };
  let successCount = 0;
  let failCount = 0;
  const usedExportNames = new Set();

  for (const pkg of selectedRows.value) {
    if (extractCancelled.value) break;
    batchExtractProgress.value.current = successCount + failCount + 1;
    const appLabel = appLabels.value[pkg]?.label || pkg;
    const baseName = buildApkExportBaseName(pkg, usedExportNames);
    const savePath = await join(dirPath, `${baseName}.apks`);
    addLog(`[${batchExtractProgress.value.current}/${selectedRows.value.length}] 正在提取: ${appLabel} (${pkg})...`);
    try {
      const actualPath = await extractApk(pkg, savePath);
      addLog(`提取成功: ${actualPath}`, 'success');
      successCount += 1;
    } catch (error) {
      addLog(`提取失败: ${appLabel} (${pkg}) - ${error}`, 'error');
      failCount += 1;
    }
  }

  isBatchExtracting.value = false;
  const wasCancelled = extractCancelled.value;
  extractCancelled.value = false;
  if (wasCancelled) ElMessage.warning(`已停止提取，已完成 ${successCount} 个`);
  else if (failCount === 0) ElMessage.success(`批量提取完成，共提取 ${successCount} 个`);
  else ElMessage.warning(`提取完成：成功 ${successCount} 个，失败 ${failCount} 个`);
}

async function handleBatchUninstall() {
  if (selectedRows.value.length === 0) return;
  try {
    await ElMessageBox.confirm(
      `即将批量卸载选中的 ${selectedRows.value.length} 个应用，此操作不可逆。`,
      '批量卸载确认',
      { confirmButtonText: '确认卸载', cancelButtonText: '取消', type: 'warning' }
    );
  } catch {
    return;
  }

  isBatchUninstalling.value = true;
  uninstallCancelled.value = false;
  batchUninstallProgress.value = { current: 0, total: selectedRows.value.length };
  let successCount = 0;
  let failCount = 0;
  const removed = [];

  for (const pkg of selectedRows.value) {
    if (uninstallCancelled.value) break;
    batchUninstallProgress.value.current = successCount + failCount + 1;
    const appLabel = appLabels.value[pkg]?.label || pkg;
    try {
      const result = await managePackage(pkg, 'uninstall');
      addLog(`卸载成功: ${appLabel} - ${result}`, 'success');
      removed.push(pkg);
      successCount += 1;
    } catch (error) {
      addLog(`卸载失败: ${appLabel} - ${error}`, 'error');
      failCount += 1;
    }
  }

  if (removed.length > 0) {
    const removedSet = new Set(removed);
    packages.value = packages.value.filter((pkg) => !removedSet.has(pkg));
    if (selectedPackage.value && removedSet.has(selectedPackage.value.package_name)) selectedPackage.value = null;
  }

  isBatchUninstalling.value = false;
  uninstallCancelled.value = false;
  selectedRows.value = [];
  if (failCount === 0) ElMessage.success(`批量卸载完成，共卸载 ${successCount} 个`);
  else ElMessage.warning(`卸载完成：成功 ${successCount} 个，失败 ${failCount} 个`);
}

async function handleBatchBlockNetwork() {
  if (selectedRows.value.length === 0) return;
  try {
    await ElMessageBox.confirm(
      `即将对选中的 ${selectedRows.value.length} 个应用禁用网络。`,
      '批量禁网确认',
      { confirmButtonText: '确认禁网', cancelButtonText: '取消', type: 'warning' }
    );
  } catch {
    return;
  }

  isBatchNetworkBlocking.value = true;
  networkBlockCancelled.value = false;
  batchNetworkBlockProgress.value = { current: 0, total: selectedRows.value.length };
  let successCount = 0;
  let failCount = 0;

  for (const pkg of selectedRows.value) {
    if (networkBlockCancelled.value) break;
    batchNetworkBlockProgress.value.current = successCount + failCount + 1;
    const appLabel = appLabels.value[pkg]?.label || pkg;
    try {
      const result = await managePackage(pkg, 'block_network');
      addLog(`禁网成功: ${appLabel} - ${result}`, 'success');
      successCount += 1;
    } catch (error) {
      addLog(`禁网失败: ${appLabel} - ${error}`, 'error');
      failCount += 1;
    }
  }

  isBatchNetworkBlocking.value = false;
  networkBlockCancelled.value = false;
  if (failCount === 0) ElMessage.success(`批量禁网完成，共处理 ${successCount} 个`);
  else ElMessage.warning(`禁网完成：成功 ${successCount} 个，失败 ${failCount} 个`);
}

async function handleBatchUnblockNetwork() {
  if (selectedRows.value.length === 0) return;
  try {
    await ElMessageBox.confirm(
      `即将对选中的 ${selectedRows.value.length} 个应用恢复联网。`,
      '批量联网确认',
      { confirmButtonText: '确认联网', cancelButtonText: '取消', type: 'info' }
    );
  } catch {
    return;
  }

  isBatchNetworkUnblocking.value = true;
  networkUnblockCancelled.value = false;
  batchNetworkUnblockProgress.value = { current: 0, total: selectedRows.value.length };
  let successCount = 0;
  let failCount = 0;

  for (const pkg of selectedRows.value) {
    if (networkUnblockCancelled.value) break;
    batchNetworkUnblockProgress.value.current = successCount + failCount + 1;
    const appLabel = appLabels.value[pkg]?.label || pkg;
    try {
      const result = await managePackage(pkg, 'unblock_network');
      addLog(`联网成功: ${appLabel} - ${result}`, 'success');
      successCount += 1;
    } catch (error) {
      addLog(`联网失败: ${appLabel} - ${error}`, 'error');
      failCount += 1;
    }
  }

  isBatchNetworkUnblocking.value = false;
  networkUnblockCancelled.value = false;
  if (failCount === 0) ElMessage.success(`批量联网完成，共处理 ${successCount} 个`);
  else ElMessage.warning(`联网完成：成功 ${successCount} 个，失败 ${failCount} 个`);
}

async function handleInstallApk() {
  try {
    const selected = await open({ multiple: false, filters: [{ name: 'APK', extensions: ['apk'] }] });
    if (selected) {
      addLog(`准备安装 APK: ${selected}`);
      await installPackage(selected, true, false);
      addLog('安装成功', 'success');
      fetchPackages();
    }
  } catch (error) {
    addLog(`安装失败: ${error}`, 'error');
  }
}

async function copyPath(path) {
  try {
    await navigator.clipboard.writeText(path);
    ElMessage({ message: '路径已复制', type: 'success', duration: 1500 });
  } catch {
    ElMessage({ message: '复制失败', type: 'error', duration: 1500 });
  }
}

function handleChangeAppType(value) {
  appType.value = value;
  fetchPackages();
}

function handleChangeStatusFilter(value) {
  statusFilter.value = value;
}

defineExpose({ refresh: fetchPackages });

onMounted(fetchPackages);
</script>

<style lang="scss" scoped>
.apps-management {
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
