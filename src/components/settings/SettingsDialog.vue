<template>
  <el-dialog
    v-model="visible"
    title="应用设置"
    width="620px"
    align-center
    class="app-dialog-shell settings-dialog"
    :close-on-click-modal="true"
    append-to-body
    destroy-on-close
  >
    <div class="settings-dialog-body">
      <SettingsPanel />
    </div>
  </el-dialog>
</template>

<script setup>
import { ref } from 'vue';
import SettingsPanel from '@/components/settings/SettingsPanel.vue';

const visible = ref(false);

function open() {
  visible.value = true;
}

defineExpose({ open });
</script>

<style lang="scss" scoped>
.settings-dialog {
  :deep(.el-dialog__header) {
    // padding: 20px 24px 10px;
  }

  :deep(.el-dialog__title) {
    font-size: 16px;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  :deep(.el-dialog__body) {
    padding: 0 24px 24px;
  }
}

.settings-dialog-body {
  max-height: 72vh;
  overflow: auto;
  padding: 0;
}
</style>
