<template>
  <el-dialog v-model="visibleProxy" :title="isEdit ? '编辑属性' : '新增属性'" width="400px" append-to-body class="app-dialog-shell sys-props-dialog">
    <el-form :model="form" label-position="top">
      <el-form-item v-if="!isEdit" label="类型">
        <el-select v-model="form.type" placeholder="请选择类型" class="type-select">
          <el-option v-for="tab in tabs" :key="tab.value" :label="tab.label" :value="tab.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="键名 (Key)">
        <el-input v-model="form.key" placeholder="请输入键名" :disabled="isEdit" />
      </el-form-item>
      <el-form-item label="取值 (Value)">
        <el-input v-model="form.value" type="textarea" :rows="3" placeholder="请输入属性值" />
      </el-form-item>
    </el-form>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="visibleProxy = false">取消</el-button>
        <el-button type="primary" @click="$emit('confirm')">确定</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
  isEdit: { type: Boolean, default: false },
  form: { type: Object, required: true },
  tabs: { type: Array, default: () => [] },
});

const emit = defineEmits(['update:visible', 'confirm']);

const visibleProxy = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});
</script>

<style lang="scss" scoped>
:deep(.sys-props-dialog) {
  .el-dialog__header {
    border-bottom: 1px solid var(--color-divider);
    padding-bottom: 15px;
  }

  .el-dialog__body {
    padding-top: 20px;
  }

  .el-form-item__label {
    font-weight: 600;
    font-size: 13px;
    color: var(--color-text-secondary);
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.type-select {
  width: 100%;
}
</style>
