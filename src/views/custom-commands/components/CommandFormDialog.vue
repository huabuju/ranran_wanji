<template>
  <el-dialog
    v-model="visibleProxy"
    :title="editingId ? '编辑指令' : '新增指令'"
    width="480px"
    append-to-body
    destroy-on-close
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-width="70px" @submit.prevent>
      <el-form-item label="名称" prop="name">
        <el-input v-model="form.name" placeholder="请输入指令名称（如：重启到恢复模式）" />
      </el-form-item>
      <el-form-item label="命令行" prop="command">
        <el-input
          v-model="form.command"
          type="textarea"
          :autosize="{ minRows: 2, maxRows: 5 }"
          placeholder="请输入完整执行命令（如：adb shell ls -al /sdcard/）"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="visibleProxy = false">取消</el-button>
        <el-button type="primary" @click="handleSave">保存</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed, ref } from 'vue';

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  editingId: {
    type: [String, Number, null],
    default: null,
  },
  form: {
    type: Object,
    required: true,
  },
  rules: {
    type: Object,
    default: () => ({}),
  },
});

const emit = defineEmits(['update:visible', 'save']);
const formRef = ref(null);

const visibleProxy = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});

async function handleSave() {
  if (!formRef.value) {
    return;
  }

  const valid = await formRef.value.validate().catch(() => false);
  if (valid !== false) {
    emit('save');
  }
}
</script>
