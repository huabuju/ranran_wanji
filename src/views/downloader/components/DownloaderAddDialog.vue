<template>
  <el-dialog v-model="visibleProxy" title="新建下载任务" width="480px" append-to-body class="app-dialog-shell downloader-dialog">
    <el-form :model="form" label-position="top">
      <el-form-item label="链接地址">
        <el-input v-model="form.url" type="textarea" :rows="3" placeholder="粘贴 HTTP/HTTPS 链接" @blur="$emit('auto-fill-file-name')" />
      </el-form-item>
      <el-form-item label="保存到">
        <el-input v-model="form.saveDir" placeholder="保存目录">
          <template #append>
            <el-button @click="$emit('select-save-dir')">选择目录</el-button>
          </template>
        </el-input>
      </el-form-item>
      <div class="advanced-toggle" @click="$emit('toggle-advanced')">
        高级选项
        <el-icon><ArrowDown v-if="!showAdvanced" /><ArrowUp v-else /></el-icon>
      </div>
      <div v-show="showAdvanced" class="advanced-area">
        <el-form-item label="下载线程">
          <el-slider v-model="form.threads" :min="1" :max="16" show-input />
        </el-form-item>
        <el-form-item label="Referer">
          <el-input v-model="form.referer" placeholder="留空则自动使用链接域名（推荐）" clearable />
          <div class="referer-tip">遇到 403 错误时，可尝试填写资源所在网页的 URL</div>
        </el-form-item>
      </div>
    </el-form>
    <template #footer>
      <el-button round @click="visibleProxy = false">取消</el-button>
      <el-button type="primary" round :loading="isAdding" @click="$emit('start-download')">立即下载</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed } from 'vue';
import { ArrowDown, ArrowUp } from '@element-plus/icons-vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
  form: { type: Object, required: true },
  showAdvanced: { type: Boolean, default: false },
  isAdding: { type: Boolean, default: false },
});

const emit = defineEmits(['update:visible', 'auto-fill-file-name', 'select-save-dir', 'toggle-advanced', 'start-download']);

const visibleProxy = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});
</script>

<style lang="scss" scoped>
.downloader-dialog {
  .advanced-toggle {
    font-size: 13px; color: var(--color-text-secondary); cursor: pointer; margin: 10px 0; display: inline-flex; align-items: center; gap: 4px;
    &:hover { color: var(--el-color-primary); }
  }
  .advanced-area {
    background: var(--surface-panel); padding: 16px; border-radius: 8px; margin-bottom: 12px; border: 1px dashed var(--color-border);
    .referer-tip { margin-top: 6px; font-size: 12px; color: var(--color-text-muted); line-height: 1.5; }
  }
}
</style>
