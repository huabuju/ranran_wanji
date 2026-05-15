<template>
  <div class="downloader-container">
    <DownloaderHeader
      v-model:active-tab="activeTab"
      :downloading-count="downloadingList.length"
      :completed-count="completedList.length"
      @cancel-all="cancelAll"
      @clear-completed="clearCompleted"
      @show-add="openAddTaskDialog()"
    />

    <DownloaderTaskList
      :active-tab="activeTab"
      :tasks="activeTab === 'downloading' ? downloadingList : completedList"
      :get-file-type="getFileType"
      :get-status-label="getStatusLabel"
      :get-status-type="getStatusType"
      :get-progress-status="getProgressStatus"
      :format-time="formatTime"
      @show-add="openAddTaskDialog()"
      @cancel-task="cancelTask"
      @open-file="openFile"
      @open-folder="openFolder"
      @retry-task="retryTask"
      @remove-task="removeTask"
    />

    <DownloaderAddDialog
      v-model:visible="showAddDialog"
      :form="addForm"
      :show-advanced="showAdvanced"
      :is-adding="isAdding"
      @auto-fill-file-name="autoFillFileName"
      @select-save-dir="selectSaveDir"
      @toggle-advanced="showAdvanced = !showAdvanced"
      @start-download="startDownload"
    />
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { join } from '@tauri-apps/api/path';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';
import { ElMessage, ElMessageBox } from 'element-plus';
import dayjs from 'dayjs';
import { DownloaderAddDialog, DownloaderHeader, DownloaderTaskList } from './components';
import { getSystemDownloadDir } from '@/utils/systemPaths';

const activeTab = ref('downloading');
const showAdvanced = ref(false);
const tasks = ref([]);
const showAddDialog = ref(false);
const isAdding = ref(false);
const defaultSaveDir = ref('');
const addForm = ref({
  url: '',
  fileName: '',
  saveDir: '',
  threads: 16,
  referer: '',
});

const downloadingList = computed(() => tasks.value.filter((item) => ['downloading', 'pending', 'paused'].includes(item.status)));
const completedList = computed(() => tasks.value.filter((item) => ['completed', 'error', 'cancelled'].includes(item.status)));

function autoFillFileName() {
  if (!addForm.value.fileName && addForm.value.url) {
    try {
      const url = new URL(addForm.value.url.trim());
      const parts = url.pathname.split('/');
      const last = decodeURIComponent(parts[parts.length - 1]);
      if (last && last.includes('.')) addForm.value.fileName = last;
    } catch {}
  }
}

function resetAddForm(prefill = {}) {
  addForm.value.url = prefill.url || '';
  addForm.value.fileName = prefill.fileName || '';
  addForm.value.saveDir = defaultSaveDir.value;
  addForm.value.threads = 16;
  addForm.value.referer = prefill.referer || '';
}

function openAddTaskDialog(prefill = {}) {
  resetAddForm(prefill);
  showAddDialog.value = true;
}

async function selectSaveDir() {
  try {
    const selected = await openDialog({ directory: true, multiple: false, title: '选择下载保存目录' });
    if (selected) addForm.value.saveDir = selected;
  } catch (error) {
    console.error('选择目录失败:', error);
  }
}

async function startDownload() {
  if (!addForm.value.url.trim()) {
    ElMessage.warning('请输入下载链接');
    return;
  }

  isAdding.value = true;
  let id = null;
  try {
    id = `dl_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
    const fileName = addForm.value.fileName.trim() || extractFileName(addForm.value.url) || 'file';
    const referer = addForm.value.referer.trim() || null;

    tasks.value.push({
      id,
      url: addForm.value.url.trim(),
      fileName,
      saveDir: addForm.value.saveDir,
      threads: addForm.value.threads,
      referer,
      status: 'pending',
      progress: 0,
      speed: '0B/s',
      downloaded: '0B',
      eta: '--',
      totalSize: '--',
      completedAt: null,
      errorMsg: null,
    });

    await invoke('start_download', {
      id,
      url: addForm.value.url.trim(),
      saveDir: addForm.value.saveDir,
      fileName,
      threads: addForm.value.threads,
      referer,
    });

    showAddDialog.value = false;
    activeTab.value = 'downloading';
    ElMessage.success('任务已提交');
    resetAddForm();
  } catch (error) {
    tasks.value = tasks.value.filter((item) => item.id !== id);
    ElMessage.error({ message: String(error), duration: 6000, showClose: true });
  } finally {
    isAdding.value = false;
  }
}

async function cancelTask(task) {
  try {
    await ElMessageBox.confirm('确定取消下载并删除已下载的文件吗？', '提示', { confirmButtonText: '确定', cancelButtonText: '取消', type: 'warning' });
    await invoke('cancel_download', { id: task.id });
  } catch {}
}

async function cancelAll() {
  try {
    await ElMessageBox.confirm('确定取消所有正在下载的任务吗？', '提示', { type: 'warning' });
    for (const task of downloadingList.value) {
      invoke('cancel_download', { id: task.id }).catch(() => {});
    }
  } catch {}
}

function retryTask(task) {
  activeTab.value = 'downloading';
  invoke('retry_download', { id: task.id }).catch((error) => ElMessage.error(`重试失败: ${error}`));
}

function removeTask(task) {
  invoke('remove_download_task', { id: task.id }).catch(() => {});
  tasks.value = tasks.value.filter((item) => item.id !== task.id);
}

async function clearCompleted() {
  try {
    await ElMessageBox.confirm('确定清空所有传输记录吗？', '提示', { type: 'warning' });
    for (const task of completedList.value) {
      invoke('remove_download_task', { id: task.id }).catch(() => {});
    }
    tasks.value = tasks.value.filter((item) => !['completed', 'error', 'cancelled'].includes(item.status));
  } catch {}
}

function openFolder(task) {
  invoke('open_download_folder', { saveDir: task.saveDir }).catch((error) => ElMessage.error(`打开失败: ${error}`));
}

async function openFile(task) {
  try {
    const filePath = await join(task.saveDir, task.fileName);
    await openPath(filePath);
  } catch (error) {
    ElMessage.error(`打开文件失败: ${error}`);
  }
}

let unlistenFn = null;

onMounted(async () => {
  defaultSaveDir.value = await getSystemDownloadDir();
  resetAddForm();

  try {
    tasks.value = await invoke('get_download_tasks');
  } catch {}

  unlistenFn = await listen('download-progress', (event) => {
    const payload = event.payload;
    const idx = tasks.value.findIndex((item) => item.id === payload.id);
    if (idx === -1) return;
    const task = tasks.value[idx];
    const isDone = ['completed', 'cancelled', 'error'].includes(payload.status);
    tasks.value[idx] = {
      ...task,
      status: payload.status,
      progress: payload.progress ?? task.progress,
      speed: isDone ? '' : (payload.speed ?? task.speed),
      downloaded: payload.downloaded ?? task.downloaded,
      eta: isDone ? '' : (payload.eta ?? task.eta),
      totalSize: payload.totalSize ?? task.totalSize,
      completedAt: payload.completedAt ?? task.completedAt,
      errorMsg: payload.errorMsg ?? task.errorMsg,
    };
  });

  const pendingUrl = sessionStorage.getItem('pending_download_url');
  if (pendingUrl) {
    openAddTaskDialog({
      url: pendingUrl,
      fileName: sessionStorage.getItem('pending_download_filename') || '',
      referer: sessionStorage.getItem('pending_download_referer') || '',
    });
    sessionStorage.removeItem('pending_download_url');
    sessionStorage.removeItem('pending_download_filename');
    sessionStorage.removeItem('pending_download_referer');
  }
});

async function refresh() {
  try {
    tasks.value = await invoke('get_download_tasks');
  } catch {}
}

defineExpose({ refresh });

onUnmounted(() => { if (unlistenFn) unlistenFn(); });

function extractFileName(url) {
  try {
    const parsed = new URL(url);
    const parts = parsed.pathname.split('/');
    return decodeURIComponent(parts[parts.length - 1]) || 'file';
  } catch {
    return 'file';
  }
}

function getFileType(filename) {
  const ext = filename.split('.').pop().toLowerCase();
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext)) return 'archive';
  if (['mp4', 'mkv', 'avi', 'mov'].includes(ext)) return 'video';
  return 'file';
}

const getStatusLabel = (status) => ({
  pending: '排队中',
  downloading: '下载中',
  completed: '已完成',
  error: '失败',
  cancelled: '已取消',
}[status] || status);

const getStatusType = (status) => ({
  downloading: '',
  pending: 'info',
  completed: 'success',
  error: 'danger',
  cancelled: 'info',
}[status] || '');

function getProgressStatus(status) {
  if (status === 'error') return 'exception';
  if (status === 'completed') return 'success';
  return '';
}

function formatTime(time) {
  if (!time) return '--';
  return dayjs(time).format('YYYY-MM-DD HH:mm');
}
</script>

<style lang="scss" scoped>
.downloader-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: transparent;
  gap: 20px;
}
</style>
