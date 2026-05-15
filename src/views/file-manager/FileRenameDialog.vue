<template>
  <el-dialog v-model="visibleProxy" title="重命名" width="380px" :close-on-click-modal="false" append-to-body>
    <el-input v-model="nameProxy" placeholder="请输入新名称" autofocus @keyup.enter="$emit('confirm')" />
    <template #footer>
      <el-button @click="visibleProxy = false">取消</el-button>
      <el-button type="primary" @click="$emit('confirm')">确定</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
  newName: { type: String, default: '' },
});

const emit = defineEmits(['update:visible', 'update:new-name', 'confirm']);

const visibleProxy = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});

const nameProxy = computed({
  get: () => props.newName,
  set: (value) => emit('update:new-name', value),
});
</script>
