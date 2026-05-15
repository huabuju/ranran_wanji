<template>
  <div class="file-manager" @contextmenu.prevent="onBackgroundContextMenu">
    <FileToolbar
      :current-path="currentPath"
      :layout-mode="layoutMode"
      :loading="loading"
      @go-up="goUp"
      @refresh="loadDir"
      @navigate="navigateTo"
      @search="onSearch"
      @set-layout="layoutMode = $event"
    />

    <FileActionBar
      :displayed-files-count="displayedFiles.length"
      :selected-count="selectedPaths.size"
      :storage-info="storageInfo"
      :format-size="formatSize"
      @push="handlePush"
      @push-dir="handlePushDir"
      @mkdir="handleMkdir"
    />

    <div class="main-body">
      <FileContentPanel
        :loading="loading"
        :err-msg="errMsg"
        :layout-mode="layoutMode"
        :displayed-files="displayedFiles"
        :selected-paths="selectedPaths"
        @refresh="loadDir"
        @navigate="navigateTo"
        @select="onSelect"
        @context-menu="onFileContextMenu"
      />

      <FloatingLog :logs="logs" @clear="logs = []" />
    </div>

    <FileContextMenu
      :visible="ctxMenu.visible"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      :target-file="ctxMenu.targetFile"
      @close="ctxMenu.visible = false"
      @pull="handlePull"
      @push="handlePush"
      @push-dir="handlePushDir"
      @rename="handleRename"
      @delete="handleDelete"
      @copy-path="handleCopyPath"
      @mkdir="handleMkdir"
      @refresh="loadDir"
    />

    <FileRenameDialog
      v-model:visible="renameDialog.visible"
      v-model:new-name="renameDialog.newName"
      @confirm="confirmRename"
    />

    <FileMkdirDialog
      v-model:visible="mkdirDialog.visible"
      v-model:name="mkdirDialog.name"
      @confirm="confirmMkdir"
    />
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { basename } from '@tauri-apps/api/path';
import { open, save } from '@tauri-apps/plugin-dialog';
import { ElMessage, ElMessageBox } from 'element-plus';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { useDeviceStore } from '@/utils/deviceStore';
import FileActionBar from './FileActionBar.vue';
import FileContentPanel from './FileContentPanel.vue';
import FileContextMenu from './FileContextMenu.vue';
import FileMkdirDialog from './FileMkdirDialog.vue';
import FileRenameDialog from './FileRenameDialog.vue';
import FileToolbar from './FileToolbar.vue';

const { selectedSerial } = useDeviceStore();

const currentPath = ref('/sdcard');
const files = ref([]);
const loading = ref(false);
const errMsg = ref('');
const layoutMode = ref('grid');
const searchQuery = ref('');
const selectedPaths = ref(new Set());
const lastSelected = ref(null);
const storageInfo = ref(null);

const logs = ref([]);
const logBodyRef = ref(null);

const ctxMenu = ref({ visible: false, x: 0, y: 0, targetFile: null });
const renameDialog = ref({ visible: false, file: null, newName: '' });
const mkdirDialog = ref({ visible: false, name: '新建文件夹' });

function addLog(msg, type = 'info') {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
  logs.value.push({ time, msg, type });
  nextTick(() => {
    if (logBodyRef.value) {
      logBodyRef.value.scrollTop = logBodyRef.value.scrollHeight;
    }
  });
}

const displayedFiles = computed(() => {
  if (!searchQuery.value.trim()) return files.value;
  const query = searchQuery.value.toLowerCase();
  return files.value.filter((file) => file.name.toLowerCase().includes(query));
});

async function loadDir() {
  loading.value = true;
  errMsg.value = '';
  selectedPaths.value = new Set();
  lastSelected.value = null;

  try {
    addLog(`进入: ${currentPath.value}`);
    const result = await invoke('adb_list_dir', {
      serial: selectedSerial.value || null,
      path: currentPath.value,
    });
    files.value = result;
    addLog(`加载完成，共 ${result.length} 项`, 'success');
    loadStorageInfo();
  } catch (error) {
    errMsg.value = String(error);
    addLog(`加载失败: ${error}`, 'error');
  } finally {
    loading.value = false;
  }
}

async function loadStorageInfo() {
  try {
    storageInfo.value = await invoke('adb_get_storage_info', {
      serial: selectedSerial.value || null,
      path: currentPath.value,
    });
  } catch {
    // 存储信息为增强能力，读取失败时静默忽略。
  }
}

function navigateTo(path) {
  currentPath.value = path;
  searchQuery.value = '';
  loadDir();
}

function goUp() {
  const parts = currentPath.value.split('/').filter(Boolean);
  if (parts.length === 0) return;
  parts.pop();
  const newPath = `/${parts.join('/')}`;
  navigateTo(newPath || '/');
}

function onSearch(query) {
  searchQuery.value = query;
}

function onSelect(newSet, file) {
  selectedPaths.value = newSet;
  if (file) lastSelected.value = file;
  else if (newSet.size === 0) lastSelected.value = null;
}

function showCtxMenu(event, targetFile) {
  const menuW = 190;
  const menuH = 200;
  let x = event.clientX;
  let y = event.clientY;

  if (x + menuW > window.innerWidth) x = window.innerWidth - menuW - 8;
  if (y + menuH > window.innerHeight) y = window.innerHeight - menuH - 8;

  ctxMenu.value = { visible: true, x, y, targetFile };
}

function onFileContextMenu(event, file) {
  showCtxMenu(event, file);
}

function onBackgroundContextMenu(event) {
  showCtxMenu(event, null);
  selectedPaths.value = new Set();
  lastSelected.value = null;
}

async function handlePull(file) {
  ctxMenu.value.visible = false;
  try {
    const savePath = await save({ defaultPath: file.name });
    if (!savePath) return;
    addLog(`正在下载: ${file.name} -> ${savePath}`);
    const result = await invoke('adb_pull_file', {
      serial: selectedSerial.value || null,
      remotePath: file.path,
      localPath: savePath,
    });
    addLog(`下载完成: ${result}`, 'success');
    ElMessage.success('下载完成');
  } catch (error) {
    addLog(`下载失败: ${error}`, 'error');
    ElMessage.error(`下载失败: ${error}`);
  }
}

async function handlePush() {
  ctxMenu.value.visible = false;
  try {
    const selected = await open({ multiple: true });
    if (!selected || selected.length === 0) return;
    const fileList = Array.isArray(selected) ? selected : [selected];
    for (const localPath of fileList) {
      const fileName = await basename(localPath);
      const remotePath = `${currentPath.value.replace(/\/$/, '')}/${fileName}`;
      addLog(`正在上传: ${fileName} -> ${remotePath}`);
      const result = await invoke('adb_push_file', {
        serial: selectedSerial.value || null,
        localPath,
        remotePath,
      });
      addLog(`上传完成: ${result}`, 'success');
    }
    ElMessage.success('上传完成');
    await loadDir();
  } catch (error) {
    addLog(`上传失败: ${error}`, 'error');
    ElMessage.error(`上传失败: ${error}`);
  }
}

async function handlePushDir() {
  ctxMenu.value.visible = false;
  try {
    const dirPath = await open({ directory: true });
    if (!dirPath) return;
    const dirName = await basename(dirPath);
    const remotePath = `${currentPath.value.replace(/\/$/, '')}/${dirName}`;
    addLog(`正在上传文件夹: ${dirName} -> ${remotePath}`);
    const result = await invoke('adb_push_file', {
      serial: selectedSerial.value || null,
      localPath: dirPath,
      remotePath,
    });
    addLog(`上传完成: ${result}`, 'success');
    ElMessage.success('文件夹上传完成');
    await loadDir();
  } catch (error) {
    addLog(`上传失败: ${error}`, 'error');
    ElMessage.error(`上传失败: ${error}`);
  }
}

async function handleDelete(file) {
  ctxMenu.value.visible = false;
  try {
    await ElMessageBox.confirm(
      `确定要删除 "${file.name}" 吗？${file.is_dir ? '（文件夹及其所有内容）' : ''}`,
      '删除确认',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消', confirmButtonClass: 'el-button--danger' }
    );
    addLog(`正在删除: ${file.path}`);
    const result = await invoke('adb_delete_item', {
      serial: selectedSerial.value || null,
      remotePath: file.path,
    });
    addLog(result, 'success');
    ElMessage.success('删除成功');
    await loadDir();
  } catch (error) {
    if (error === 'cancel') return;
    addLog(`删除失败: ${error}`, 'error');
    ElMessage.error(`删除失败: ${error}`);
  }
}

function handleRename(file) {
  ctxMenu.value.visible = false;
  renameDialog.value = { visible: true, file, newName: file.name };
}

async function confirmRename() {
  const { file, newName } = renameDialog.value;
  if (!file || !newName.trim() || newName === file.name) {
    renameDialog.value.visible = false;
    return;
  }
  const newPath = `${currentPath.value.replace(/\/$/, '')}/${newName.trim()}`;
  try {
    addLog(`重命名: ${file.name} -> ${newName}`);
    const result = await invoke('adb_rename_item', {
      serial: selectedSerial.value || null,
      oldPath: file.path,
      newPath,
    });
    addLog(result, 'success');
    ElMessage.success('重命名成功');
    renameDialog.value.visible = false;
    await loadDir();
  } catch (error) {
    addLog(`重命名失败: ${error}`, 'error');
    ElMessage.error(`重命名失败: ${error}`);
  }
}

function handleMkdir() {
  ctxMenu.value.visible = false;
  mkdirDialog.value = { visible: true, name: '新建文件夹' };
}

async function confirmMkdir() {
  const name = mkdirDialog.value.name.trim();
  if (!name) return;
  const remotePath = `${currentPath.value.replace(/\/$/, '')}/${name}`;
  try {
    addLog(`正在创建文件夹: ${remotePath}`);
    const result = await invoke('adb_mkdir', {
      serial: selectedSerial.value || null,
      remotePath,
    });
    addLog(result, 'success');
    ElMessage.success('创建成功');
    mkdirDialog.value.visible = false;
    await loadDir();
  } catch (error) {
    addLog(`创建失败: ${error}`, 'error');
    ElMessage.error(`创建失败: ${error}`);
  }
}

async function handleCopyPath(file) {
  ctxMenu.value.visible = false;
  try {
    await navigator.clipboard.writeText(file.path);
    ElMessage.success('路径已复制到剪贴板');
  } catch {
    ElMessage.error('复制失败');
  }
}

function formatSize(bytes) {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

onMounted(loadDir);

defineExpose({ refresh: loadDir });
</script>

<style lang="scss" scoped>
.file-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 10px;
  background: transparent;
  overflow: hidden;
}

.main-body {
  flex: 1;
  display: flex;
  gap: 10px;
  overflow: hidden;
  min-height: 0;
}
</style>
