<template>
  <div class="custom-commands-container">
    <CustomCommandsToolbar
      :is-batch-mode="isBatchMode"
      :custom-commands-count="customCommandsCount"
      :selected-count="selectedCount"
      :is-all-custom-selected="isAllCustomSelected"
      @enter-batch-mode="enterBatchMode"
      @exit-batch-mode="exitBatchMode"
      @toggle-select-all="toggleSelectAllCustom"
      @batch-delete="handleBatchDelete"
      @add-command="openEditDialog()"
    />

    <CommandCardsGrid
      :commands="commands"
      :is-batch-mode="isBatchMode"
      :is-selected="isSelected"
      @add-command="openEditDialog()"
      @sort-end="saveToStorage"
      @card-click="handleCardClick"
      @edit-command="openEditDialog"
      @delete-command="deleteCommand"
      @execute-command="executeCommand"
    />

    <CommandFormDialog
      v-model:visible="dialogVisible"
      :editing-id="editingId"
      :form="form"
      :rules="rules"
      @save="saveCommand"
    />

    <FloatingLog ref="floatingLogRef" :logs="logs" @clear="logs = []" />
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import FloatingLog from '@/components/common/FloatingLog.vue';
import { runAdb, runAdbShell, runFastboot } from '@/api/device';
import { useDeviceStore } from '@/utils/deviceStore';
import { CommandCardsGrid, CommandFormDialog, CustomCommandsToolbar } from './components';

const { selectedDevice } = useDeviceStore();

const LOCAL_STORAGE_KEY = 'toolkit_custom_commands';
const ORDER_KEY = 'toolkit_custom_commands_order';

const commands = ref([]);
const isBatchMode = ref(false);
const selectedIds = ref([]);
const dialogVisible = ref(false);
const editingId = ref(null);
const form = reactive({
  name: '',
  command: '',
});
const logs = ref([]);
const floatingLogRef = ref(null);

const BUILTIN_COMMANDS = [
  { id: 'builtin_battery', name: '查看设备电池详细状态', command: 'adb shell dumpsys battery', isBuiltIn: true, needConnectDevices: true },
  { id: 'builtin_shizuku', name: '激活 Shizuku (自动寻址)', command: 'adb shell .../libshizuku.so', isBuiltIn: true, needConnectDevices: true },
  { id: 'builtin_scene', name: '激活 Scene', command: 'adb shell sh /storage/emulated/0/Android/data/com.omarea.vtools/up.sh', isBuiltIn: true, needConnectDevices: true },
  // { id: 'builtin_mdns', name: '显示 MDNS 服务', command: 'adb mdns services', isBuiltIn: true, needConnectDevices: false },
];

const customCommandsCount = computed(() => commands.value.filter((item) => !item.isBuiltIn).length);
const allCustomIds = computed(() => commands.value.filter((item) => !item.isBuiltIn).map((item) => item.id));
const selectedCount = computed(() => selectedIds.value.length);
const isAllCustomSelected = computed(() => customCommandsCount.value > 0 && selectedCount.value === customCommandsCount.value);

const rules = {
  name: [{ required: true, message: '请输入指令名称', trigger: 'blur' }],
  command: [{ required: true, message: '请输入完整命令', trigger: 'blur' }],
};

const executingId = ref(null);
const isRunning = ref(false);

function addLog(content, type = 'info') {
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
  logs.value.push({ time, content, type });
}

function openLogsPanel() {
  floatingLogRef.value?.open?.();
}

function isSelected(id) {
  return selectedIds.value.includes(id);
}

function enterBatchMode() {
  if (customCommandsCount.value === 0) {
    return;
  }
  isBatchMode.value = true;
  selectedIds.value = [];
}

function exitBatchMode() {
  isBatchMode.value = false;
  selectedIds.value = [];
}

function handleCardClick(item) {
  if (!isBatchMode.value || item.isBuiltIn) {
    return;
  }
  toggleSelection(item);
}

function toggleSelection(item) {
  if (item.isBuiltIn) {
    return;
  }

  if (isSelected(item.id)) {
    selectedIds.value = selectedIds.value.filter((id) => id !== item.id);
  } else {
    selectedIds.value = [...selectedIds.value, item.id];
  }
}

function toggleSelectAllCustom() {
  selectedIds.value = isAllCustomSelected.value ? [] : [...allCustomIds.value];
}

async function handleBatchDelete() {
  if (selectedCount.value === 0) {
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedCount.value} 条自定义指令吗？此操作不可恢复。`,
      '批量删除确认',
      {
        confirmButtonText: '确认删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
  } catch {
    return;
  }

  const removeSet = new Set(selectedIds.value);
  const deletedCount = selectedIds.value.length;
  commands.value = commands.value.filter((item) => item.isBuiltIn || !removeSet.has(item.id));
  saveToStorage();
  selectedIds.value = [];
  isBatchMode.value = false;
  ElMessage.success(`已删除 ${deletedCount} 条指令`);
}

function syncSelectedIds() {
  const validIds = new Set(allCustomIds.value);
  selectedIds.value = selectedIds.value.filter((id) => validIds.has(id));
  if (selectedIds.value.length === 0 && isBatchMode.value && customCommandsCount.value === 0) {
    isBatchMode.value = false;
  }
}

onMounted(() => {
  loadCommands();
});

function loadCommands() {
  const data = localStorage.getItem(LOCAL_STORAGE_KEY);
  const orderData = localStorage.getItem(ORDER_KEY);

  let userCommands = [];
  if (data) {
    try {
      userCommands = JSON.parse(data).filter((item) => !item.isBuiltIn);
    } catch (error) {
      console.error('Failed to parse custom commands', error);
    }
  }

  const allMap = {};
  BUILTIN_COMMANDS.forEach((item) => {
    allMap[item.id] = item;
  });
  userCommands.forEach((item) => {
    allMap[item.id] = item;
  });

  let orderIds = [];
  if (orderData) {
    try {
      orderIds = JSON.parse(orderData);
    } catch (error) {
      console.error('Failed to parse command order', error);
    }
  }

  const sorted = [];
  const seenIds = new Set();

  orderIds.forEach((id) => {
    if (allMap[id] && !seenIds.has(id)) {
      sorted.push(allMap[id]);
      seenIds.add(id);
    }
  });

  Object.keys(allMap).forEach((id) => {
    if (!seenIds.has(id)) {
      sorted.push(allMap[id]);
    }
  });

  commands.value = sorted;
  syncSelectedIds();
}

function saveToStorage() {
  const orderIds = [...new Set(commands.value.map((item) => item.id))];
  localStorage.setItem(ORDER_KEY, JSON.stringify(orderIds));

  const userCommands = commands.value.filter((item) => !item.isBuiltIn);
  localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(userCommands));
  syncSelectedIds();
}

function openEditDialog(item = null) {
  if (item) {
    editingId.value = item.id;
    form.name = item.name;
    form.command = item.command;
  } else {
    editingId.value = null;
    form.name = '';
    form.command = '';
  }
  dialogVisible.value = true;
}

function saveCommand() {
  if (editingId.value) {
    const index = commands.value.findIndex((item) => item.id === editingId.value);
    if (index !== -1) {
      commands.value[index] = {
        ...commands.value[index],
        id: editingId.value,
        name: form.name,
        command: form.command.trim(),
      };
    }
  } else {
    const newId = `custom_${Date.now()}_${Math.floor(Math.random() * 1000)}`;
    commands.value.push({
      id: newId,
      name: form.name,
      command: form.command.trim(),
    });
  }

  saveToStorage();
  dialogVisible.value = false;
  ElMessage.success(editingId.value ? '修改成功' : '新增成功');
}

function deleteCommand(id) {
  const targetId = typeof id === 'object' ? id.id : id;
  commands.value = commands.value.filter((item) => item.id !== targetId);
  saveToStorage();
  ElMessage.success('已删除');
}

function parseCommandArgs(cmdString) {
  const args = [];
  let currentArg = '';
  let inDoubleQ = false;
  let inSingleQ = false;
  let hasArg = false;

  for (let index = 0; index < cmdString.length; index += 1) {
    const char = cmdString[index];

    if (char === '\\' && !inSingleQ && index + 1 < cmdString.length) {
      const nextChar = cmdString[index + 1];
      if (nextChar === '"' || nextChar === "'" || nextChar === ' ' || nextChar === '\\') {
        currentArg += nextChar;
        hasArg = true;
        index += 1;
        continue;
      }

      currentArg += char;
      hasArg = true;
      continue;
    }

    if (char === '"' && !inSingleQ) {
      inDoubleQ = !inDoubleQ;
      hasArg = true;
      continue;
    }

    if (char === "'" && !inDoubleQ) {
      inSingleQ = !inSingleQ;
      hasArg = true;
      continue;
    }

    if (/\s/.test(char) && !inDoubleQ && !inSingleQ) {
      if (hasArg) {
        args.push(currentArg);
        currentArg = '';
        hasArg = false;
      }
      continue;
    }

    currentArg += char;
    hasArg = true;
  }

  if (hasArg) {
    args.push(currentArg);
  }

  return args;
}

async function executeCommand(item) {
  if (!selectedDevice.value && item.needConnectDevices) {
    ElMessage.warning('请先连接设备后再执行指令');
    return;
  }

  executingId.value = item.id;
  isRunning.value = true;

  if (item.id === 'builtin_shizuku') {
    await handleActivateShizuku(item);
    return;
  }

  addLog(`执行: ${item.name}`, 'info');
  ElMessage.success(`正在执行: ${item.name}`);

  try {
    let result = '';
    const commandString = item.command.trim();

    if (commandString.startsWith('adb shell ')) {
      result = await runAdbShell(parseCommandArgs(commandString.substring(10)));
    } else if (commandString.startsWith('fastboot ')) {
      result = await runFastboot(parseCommandArgs(commandString.substring(9)));
    } else if (commandString.startsWith('adb ')) {
      result = await runAdb(parseCommandArgs(commandString.substring(4)));
    } else {
      throw new Error('不支持的命令格式，必须以 adb、adb shell 或 fastboot 开头');
    }

    addLog(result || '执行成功（无输出内容）', 'success');
  } catch (error) {
    addLog(`执行失败:\n${error.toString()}`, 'error');
    ElMessage.error(`执行 ${item.name} 时失败`);
  } finally {
    openLogsPanel();
    isRunning.value = false;
    executingId.value = null;
  }
}

async function handleActivateShizuku(item) {
  addLog(`开始执行: ${item.name}`, 'info');
  try {
    addLog('正在查找 Shizuku 安装目录...', 'info');
    const pathOutput = await runAdbShell(['pm', 'path', 'moe.shizuku.privileged.api']);
    const pathLine = pathOutput.split('\n').find((line) => line.includes('package:'));
    if (!pathLine) {
      throw new Error('手机未安装 Shizuku，请先下载并至少打开一次应用');
    }

    const apkPath = pathLine.replace('package:', '').trim();
    const baseDir = apkPath.substring(0, apkPath.lastIndexOf('/'));
    addLog(`定位成功: ${baseDir}`, 'success');

    addLog('正在识别 CPU 架构...', 'info');
    const archOutput = (await runAdbShell(['uname', '-m'])).trim();
    let arch = 'arm64';
    if (archOutput === 'aarch64') arch = 'arm64';
    else if (archOutput.includes('arm')) arch = 'arm';
    else if (archOutput.includes('x86_64')) arch = 'x86_64';
    else if (archOutput.includes('x86')) arch = 'x86';
    addLog(`识别成功: ${archOutput} -> 使用 lib/${arch}`, 'success');

    const libPath = `${baseDir}/lib/${arch}/libshizuku.so`;
    addLog(`准备发送激活命令: ${libPath}`, 'info');

    let result = '';
    try {
      result = await runAdbShell([libPath]);
    } catch {
      addLog('直接启动失败，尝试使用 sh 引导启动...', 'info');
      result = await runAdbShell(['sh', libPath]);
    }

    addLog(result || '激活完成（该命令通常无回显，请在手机端确认状态）', 'success');
    ElMessage.success('激活指令已发送');
  } catch (error) {
    const message = error.toString();
    addLog(`激活失败: ${message}`, 'error');
    ElMessage.error(`激活失败: ${message}`);
  } finally {
    openLogsPanel();
    isRunning.value = false;
    executingId.value = null;
  }
}

defineExpose({ refresh: loadCommands });
</script>

<style lang="scss" scoped>
.custom-commands-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 12px;
  background-color: transparent;
}
</style>
