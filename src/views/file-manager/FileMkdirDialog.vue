<template>
  <el-dialog v-model="visibleProxy" title="新建文件夹" width="380px" :close-on-click-modal="false" append-to-body>
    <el-input v-model="nameProxy" placeholder="请输入文件夹名称" autofocus @keyup.enter="$emit('confirm')" />
    <template #footer>
      <el-button @click="visibleProxy = false">取消</el-button>
      <el-button type="primary" @click="$emit('confirm')">创建</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
  name: { type: String, default: '' },
});

const emit = defineEmits(['update:visible', 'update:name', 'confirm']);

const visibleProxy = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});

const nameProxy = computed({
  get: () => props.name,
  set: (value) => emit('update:name', value),
});
</script>
