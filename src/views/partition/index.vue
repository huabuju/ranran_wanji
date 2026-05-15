<template>
  <div class="partition-management">
    <PartitionToolbar v-model:search-query="searchQuery" :total="totalPartitions" @scan="handleScan" />

    <PartitionFilterBar
      :filters="filters"
      :current-filter="currentFilter"
      :selected-count="selectedRows.length"
      :device-mode="deviceMode"
      :is-batch-extracting="isBatchExtracting"
      :is-batch-formatting="isBatchFormatting"
      :batch-extract-progress="batchExtractProgress"
      :batch-format-progress="batchFormatProgress"
      @change-filter="currentFilter = $event"
      @batch-extract="handleBatchExtract"
      @batch-format="handleBatchFormat"
      @cancel-extract="cancelBatchExtract"
      @cancel-format="cancelBatchFormat"
    />

    <div class="main-split-container">
      <PartitionTablePanel
        :data="filteredData"
        :loading="loading"
        :device-mode="deviceMode"
        :table-row-class-name="tableRowClassName"
        @selection-change="handleSelectionChange"
        @flash="handleFlash"
      />

      <FloatingLog :logs="logs" @clear="clearLogs" />
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue';
import { join } from '@tauri-apps/api/path';
import { ElMessage, ElMessageBox } from 'element-plus';
import { open, save } from '@tauri-apps/plugin-dialog';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { PartitionFilterBar, PartitionTablePanel, PartitionToolbar } from './components';
import { checkFileExists, fetchDeviceState, fetchPartitionInfo, runAdb, runAdbShell, runFastboot } from '@/api/device';

const searchQuery = ref('');
const currentFilter = ref('all');
const filters = [
  { label: '全部', value: 'all' },
  { label: 'Super', value: 'super' },
  { label: 'VBMeta', value: 'vbmeta' },
  { label: 'Boot', value: 'boot' },
];

const loading = ref(false);
const tableData = ref([]);
const deviceMode = ref('unknown');
const logs = ref([]);
const selectedRows = ref([]);

const isBatchExtracting = ref(false);
const isBatchFormatting = ref(false);
const extractCancelled = ref(false);
const formatCancelled = ref(false);
const batchExtractProgress = ref({ current: 0, total: 0 });
const batchFormatProgress = ref({ current: 0, total: 0 });

const cancelBatchExtract = () => { extractCancelled.value = true; };
const cancelBatchFormat = () => { formatCancelled.value = true; };

const filteredData = computed(() => {
  let result = tableData.value;
  if (currentFilter.value !== 'all') result = result.filter((item) => item.name.includes(currentFilter.value));
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter((item) => item.name.toLowerCase().includes(query) || item.path.toLowerCase().includes(query));
  }
  return result;
});

const totalPartitions = computed(() => tableData.value.length);

function addLog(content, type = 'info', tag = 'SYS') {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
  logs.value.push({ time, content, type, tag });
}

function clearLogs() {
  logs.value = [];
  addLog('日志已清空', 'info');
}

function getErrorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

async function fetchPartitions() {
  loading.value = true;
  try {
    addLog('正在刷新分区列表...', 'info', 'DEV');
    deviceMode.value = await fetchDeviceState();
    const data = await fetchPartitionInfo();
    tableData.value = data.map((item, index) => ({
      id: index + 1,
      name: item.name,
      path: item.block_device,
      fs: item.fs_type || '未知',
      statusType: 'normal',
      statusText: '200',
    }));
    addLog(`成功扫描到 ${data.length} 个分区`, 'success', 'DEV');
  } catch (error) {
    tableData.value = [];
    addLog(`获取分区表失败: ${getErrorMessage(error)}`, 'error', 'DEV');
  } finally {
    loading.value = false;
  }
}

function handleScan() {
  fetchPartitions();
  ElMessage.success('正在执行深度扫描...');
}

function tableRowClassName({ row }) {
  return row.statusType === 'error' ? 'error-row' : '';
}

function handleSelectionChange(selection) {
  selectedRows.value = selection;
}

async function extractPartitionCore(row, filePath) {
  const tmpDevicePath = `/sdcard/${row.name}.img`;
  addLog(`执行命令: dd if=${row.path} of=${tmpDevicePath}`, 'info', 'ADB');
  const ddResult = await runAdbShell(['su', '-c', `dd if=${row.path} of=${tmpDevicePath}`]);
  addLog(ddResult || 'dd 执行完成', 'success', 'ADB');
  await runAdb(['pull', tmpDevicePath, filePath]);
  await runAdbShell(['rm', tmpDevicePath]);
  return checkFileExists(filePath);
}

async function formatPartitionCore(row) {
  const eraseResult = await runFastboot(['erase', row.name]);
  const isFailed = eraseResult.toUpperCase().includes('FAILED');
  const isSuccess = eraseResult.toUpperCase().includes('OKAY') || eraseResult.toUpperCase().includes('FINISHED');
  return { eraseResult, isFailed, isSuccess };
}

async function handleFlash(row) {
  try {
    const selected = await open({ multiple: false, filters: [{ name: 'Image', extensions: ['img', 'bin'] }] });
    if (!selected) return;
    addLog(`开始刷入分区 ${row.name} <- ${selected}`, 'info', 'OP');
    loading.value = true;
    const flashResult = await runFastboot(['flash', row.name, selected]);
    const isFailed = flashResult.toUpperCase().includes('FAILED');
    const isSuccess = flashResult.toUpperCase().includes('OKAY') || flashResult.toUpperCase().includes('FINISHED');
    if (isFailed || !isSuccess) {
      addLog(`刷入失败: ${flashResult}`, 'error', 'FB');
      ElMessage.error(`分区 ${row.name} 刷入失败`);
    } else {
      addLog(flashResult, 'success', 'FB');
      ElMessage.success(`分区 ${row.name} 刷入成功`);
    }
  } catch (error) {
    if (error !== 'cancel') ElMessage.error(`刷入失败: ${String(error)}`);
  } finally {
    loading.value = false;
  }
}

async function handleBatchExtract() {
  if (selectedRows.value.length === 0) return;
  const state = await fetchDeviceState();
  if (state !== 'device') {
    ElMessage.error('设备未处于 ADB 模式，无法提取');
    return;
  }
  try {
    await ElMessageBox.confirm(`即将批量提取 ${selectedRows.value.length} 个分区，请选择保存目录。`, '批量提取确认', {
      confirmButtonText: '选择目录',
      cancelButtonText: '取消',
      type: 'info',
    });
  } catch {
    return;
  }
  const dirPath = await open({ directory: true, multiple: false });
  if (!dirPath) return;

  isBatchExtracting.value = true;
  extractCancelled.value = false;
  batchExtractProgress.value = { current: 0, total: selectedRows.value.length };
  loading.value = true;
  let successCount = 0;
  let failCount = 0;

  for (const row of selectedRows.value) {
    if (extractCancelled.value) break;
    batchExtractProgress.value.current = successCount + failCount + 1;
    try {
      const filePath = await join(dirPath, `${row.name}.img`);
      const isExist = await extractPartitionCore(row, filePath);
      if (isExist) successCount += 1;
      else failCount += 1;
    } catch {
      failCount += 1;
    }
  }

  isBatchExtracting.value = false;
  loading.value = false;
  extractCancelled.value = false;
  if (failCount === 0) ElMessage.success(`批量提取完成，共提取 ${successCount} 个分区`);
  else ElMessage.warning(`提取完成：成功 ${successCount} 个，失败 ${failCount} 个`);
}

async function handleBatchFormat() {
  if (selectedRows.value.length === 0) return;
  try {
    await ElMessageBox.confirm(`即将批量格式化选中的 ${selectedRows.value.length} 个分区，此操作不可逆。`, '批量格式化确认', {
      confirmButtonText: '确认格式化',
      cancelButtonText: '取消',
      type: 'warning',
    });
  } catch {
    return;
  }

  isBatchFormatting.value = true;
  formatCancelled.value = false;
  batchFormatProgress.value = { current: 0, total: selectedRows.value.length };
  loading.value = true;
  let successCount = 0;
  let failCount = 0;

  for (const row of selectedRows.value) {
    if (formatCancelled.value) break;
    batchFormatProgress.value.current = successCount + failCount + 1;
    try {
      const { isFailed, isSuccess } = await formatPartitionCore(row);
      if (isFailed || !isSuccess) failCount += 1;
      else successCount += 1;
    } catch {
      failCount += 1;
    }
  }

  isBatchFormatting.value = false;
  loading.value = false;
  formatCancelled.value = false;
  if (failCount === 0) ElMessage.success(`批量格式化完成，共处理 ${successCount} 个分区`);
  else ElMessage.warning(`格式化完成：成功 ${successCount} 个，失败 ${failCount} 个`);
}

defineExpose({ refresh: fetchPartitions });

onMounted(fetchPartitions);
</script>

<style lang="scss" scoped>
.partition-management {
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
