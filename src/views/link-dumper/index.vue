<template>
  <div
    class="link-dumper"
    v-loading="extracting"
    element-loading-text="正在提取分区，请耐心等待..."
    element-loading-background="var(--surface-overlay)"
    element-loading-svg-view-box="-10, -10, 50, 50"
  >
    <LinkDumperToolbar
      v-model:payload-path="payloadPath"
      :extracting="extracting"
      :total="partitions.length"
      @select-file="selectLocalFile"
      @show-help="helpDialogVisible = true"
    />

    <LinkDumperActionBar
      :fetching="fetching"
      :extracting="extracting"
      :selected-count="selectedPartitions.length"
      :total="partitions.length"
      :output-dir="outputDir"
      @fetch="fetchPartitions"
      @extract-selected="handleExtractSelected"
      @extract-all="handleExtractAll"
      @select-output="selectOutputDir"
    />

    <div class="main-split-container">
      <LinkDumperTablePanel
        :fetching="fetching"
        :partitions="partitions"
        :extracted-list="extractedList"
        :check-selectable="checkSelectable"
        :get-partition-name="getPartitionName"
        :sort-by-size="sortBySize"
        @selection-change="handleSelectionChange"
      />

      <FloatingLog :logs="logs" @clear="logs = []" />
    </div>

    <LinkDumperHelpDialog v-model:visible="helpDialogVisible" />
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { ElMessage } from 'element-plus';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { getSystemDownloadDir } from '@/utils/systemPaths';
import {
  LinkDumperActionBar,
  LinkDumperHelpDialog,
  LinkDumperTablePanel,
  LinkDumperToolbar,
} from './components';
import { extractPayloadPartitions, listPayloadPartitions } from '@/api/payload';

const payloadPath = ref('');
const helpDialogVisible = ref(false);
const outputDir = ref('');
const partitions = ref([]);
const selectedPartitions = ref([]);
const extractedList = ref([]);
const fetching = ref(false);
const extracting = ref(false);
const logs = ref([]);

let unlistenLog = null;

function addLog(content, type = 'info') {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
  logs.value.push({ time, content, type });
}

async function selectLocalFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Payload Binary', extensions: ['bin', 'zip'] }],
    });
    if (selected) {
      payloadPath.value = selected;
    }
  } catch (error) {
    addLog(`选择文件失败: ${error}`, 'error');
  }
}

async function selectOutputDir() {
  try {
    const selected = await open({ multiple: false, directory: true });
    if (selected) {
      outputDir.value = selected;
    }
  } catch (error) {
    addLog(`选择目录失败: ${error}`, 'error');
  }
}

async function fetchPartitions() {
  if (!payloadPath.value) {
    ElMessage.warning('请先输入或选择 Payload 文件路径');
    return;
  }

  fetching.value = true;
  partitions.value = [];
  try {
    addLog(`正在获取分区列表: ${payloadPath.value}...`);
    const response = await listPayloadPartitions(payloadPath.value);
    partitions.value = response.partitions;
    addLog(`成功获取 ${response.partitions.length} 个分区`, 'success');
    if (response.metadata) {
      addLog('--- Payload Metadata ---', 'warning');
      addLog(JSON.stringify(response.metadata, null, 2), 'info');
      addLog('------------------------', 'warning');
    }
  } catch (error) {
    addLog(`获取列表失败: ${error}`, 'error');
    ElMessage.error(`获取列表失败: ${error}`);
  } finally {
    fetching.value = false;
  }
}

function getPartitionName(partitionName) {
  const segments = partitionName.split('/');
  return segments[segments.length - 1];
}

function sortBySize(a, b) {
  if (a.size !== undefined && b.size !== undefined) return Number(a.size) - Number(b.size);
  const parseSize = (str) => {
    if (!str) return 0;
    const match = str.match(/([0-9.]+)\s*([a-zA-Z]*)/);
    if (!match) return 0;
    const value = parseFloat(match[1]);
    const unit = match[2].toUpperCase();
    const multipliers = { B: 1, KB: 1024, MB: 1048576, GB: 1073741824, TB: 1099511627776 };
    return value * (multipliers[unit] || 1);
  };
  return parseSize(a.size_readable) - parseSize(b.size_readable);
}

function handleSelectionChange(value) {
  selectedPartitions.value = value;
}

function checkSelectable() {
  return true;
}

async function handleExtractSelected() {
  if (!selectedPartitions.value.length) return;
  if (!outputDir.value) {
    ElMessage.warning('请先选择输出目录');
    return;
  }

  extracting.value = true;
  const targetPartitions = selectedPartitions.value.map((item) => item.partition_name);
  try {
    addLog(`开始提取选中的 ${targetPartitions.length} 个分区...`);
    await extractPayloadPartitions(payloadPath.value, targetPartitions, outputDir.value);
    addLog('提取任务启动成功', 'success');
    selectedPartitions.value.forEach((item) => {
      if (!extractedList.value.includes(item.partition_name)) extractedList.value.push(item.partition_name);
    });
  } catch (error) {
    addLog(`提取任务失败: ${error}`, 'error');
    ElMessage.error(`提取任务失败: ${error}`);
  } finally {
    extracting.value = false;
  }
}

async function handleExtractAll() {
  if (!partitions.value.length) return;
  if (!outputDir.value) {
    ElMessage.warning('请先选择输出目录');
    return;
  }

  extracting.value = true;
  const allNames = partitions.value.map((item) => item.partition_name);
  try {
    addLog(`开始提取全部 ${allNames.length} 个分区...`);
    await extractPayloadPartitions(payloadPath.value, allNames, outputDir.value);
    addLog('全量提取任务启动成功', 'success');
    partitions.value.forEach((item) => {
      if (!extractedList.value.includes(item.partition_name)) extractedList.value.push(item.partition_name);
    });
  } catch (error) {
    addLog(`全量提取失败: ${error}`, 'error');
    ElMessage.error(`全量提取失败: ${error}`);
  } finally {
    extracting.value = false;
  }
}

onMounted(async () => {
  outputDir.value = await getSystemDownloadDir();
  unlistenLog = await listen('payload-log', (event) => {
    const payload = event.payload;
    addLog(payload.content, payload.log_type);
  });
});

function refresh() {
  if (payloadPath.value) fetchPartitions();
}

defineExpose({ refresh });

onUnmounted(() => {
  if (unlistenLog) {
    unlistenLog();
  }
});
</script>

<style lang="scss" scoped>
.link-dumper {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 16px;
  background-color: transparent;
  position: relative;

  :deep(.el-loading-mask) {
    border-radius: var(--radius-md);
    backdrop-filter: blur(4px);
    z-index: 100;
  }
}

.main-split-container {
  flex: 1;
  display: flex;
  overflow: hidden;
  gap: 20px;
  background-color: transparent;
}
</style>
