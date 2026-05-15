<template>
  <div class="sys-props-container">
    <SysPropsToolbar v-model:search-query="searchQuery" :loading="loading" @refresh="handleRefresh" @add="openAddDialog" />
    <SysPropsTabs :tabs="tabs" :current-tab="currentTab" :count="filteredData.length" @change="handleTabChange" />
    <SysPropsTable
      :columns="columns"
      :data="filteredData"
      :loading="loading"
      :current-tab="currentTab"
      @copy="handleCopy"
      @edit="openEditDialog"
      @delete="handleDelete"
    />
    <SysPropsDialog v-model:visible="dialogVisible" :is-edit="isEdit" :form="form" :tabs="tabs" @confirm="handleConfirm" />
    <SysPropsNotice :visible="currentTab === 'getprop'" />
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { fetchSysProps, setSysProp, deleteSysProp } from '@/api/device';
import { useDeviceStore } from '@/utils/deviceStore';
import { SysPropsDialog, SysPropsNotice, SysPropsTable, SysPropsTabs, SysPropsToolbar } from './components';

const { isConnected } = useDeviceStore();

const loading = ref(false);
const searchQuery = ref('');
const currentTab = ref('getprop');
const propList = ref([]);
const dialogVisible = ref(false);
const isEdit = ref(false);
const form = ref({
  type: '',
  key: '',
  value: '',
});

const tabs = [
  { label: '系统属性', value: 'getprop', icon: 'settings' },
  { label: '系统设置', value: 'system', icon: 'controller' },
  { label: '全局设置', value: 'global', icon: 'overview' },
  { label: '安全设置', value: 'secure', icon: 'key' },
];

const columns = computed(() => [
  { key: 'key', dataKey: 'key', title: '键名 (Key)', width: 380, flexGrow: 0, flexShrink: 0 },
  { key: 'value', dataKey: 'value', title: '取值 (Value)', width: 380, flexGrow: 1, flexShrink: 1 },
  { key: 'operations', title: '操作', width: 180, align: 'center' },
]);

const filteredData = computed(() => {
  if (!searchQuery.value) {
    return propList.value || [];
  }
  const query = searchQuery.value.toLowerCase();
  return propList.value.filter((item) => item.key.toLowerCase().includes(query) || (item.value && item.value.toLowerCase().includes(query)));
});

async function handleRefresh() {
  if (!isConnected.value) return;

  loading.value = true;
  try {
    const data = await fetchSysProps(currentTab.value);
    propList.value = data || [];
  } catch (error) {
    ElMessage.error(`获取属性失败: ${error}`);
    propList.value = [];
  } finally {
    loading.value = false;
  }
}

function handleTabChange(value) {
  currentTab.value = value;
  handleRefresh();
}

async function handleCopy(row) {
  try {
    await navigator.clipboard.writeText(`${row.key}: ${row.value}`);
    ElMessage.success('已复制到剪贴板');
  } catch {
    ElMessage.error('复制失败');
  }
}

function openAddDialog() {
  isEdit.value = false;
  form.value = { type: currentTab.value, key: '', value: '' };
  dialogVisible.value = true;
}

function openEditDialog(row) {
  isEdit.value = true;
  form.value = { type: currentTab.value, key: row.key, value: row.value };
  dialogVisible.value = true;
}

async function handleConfirm() {
  if (!form.value.key || (!form.value.value && form.value.value !== '')) {
    ElMessage.warning('键名和取值不能为空');
    return;
  }

  try {
    const targetType = form.value.type;
    await setSysProp(targetType, form.value.key, form.value.value);
    currentTab.value = targetType;
    ElMessage.success(isEdit.value ? '修改成功' : '新增成功');
    dialogVisible.value = false;
    await handleRefresh();
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`);
  }
}

async function handleDelete(row) {
  const actionText = currentTab.value === 'getprop' ? '清空' : '删除';
  try {
    await ElMessageBox.confirm(
      `确定要${actionText}属性 "${row.key}" 吗？`,
      '确认操作',
      {
        confirmButtonClass: 'el-button--danger',
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    await deleteSysProp(currentTab.value, row.key);
    ElMessage.success(`${actionText}成功`);
    handleRefresh();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`操作失败: ${error}`);
    }
  }
}

onMounted(() => {
  if (isConnected.value) {
    handleRefresh();
  }
});

watch(isConnected, (value) => {
  if (value) handleRefresh();
  else propList.value = [];
});

defineExpose({ refresh: handleRefresh });
</script>

<style lang="scss" scoped>
.sys-props-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}
</style>
