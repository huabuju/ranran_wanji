<template>
  <el-dialog v-model="visibleProxy" title="镜像提取 / 使用指南" width="560px" class="app-dialog-shell help-dialog" :close-on-click-modal="true">
    <div class="help-content">
      <div class="help-section">
        <div class="help-section-title"><span class="help-section-icon">说明</span> 功能介绍</div>
        <p class="help-desc">
          本页面用于解析 Android 固件包中的分区镜像，支持直接选择 <code>payload.bin</code>，
          也支持包含分区信息的 <code>.zip</code> 压缩包，以及通过 URL 在线解析并提取分区镜像。
        </p>
      </div>

      <div class="help-section">
        <div class="help-section-title"><span class="help-section-icon">步骤</span> 使用步骤</div>
        <ol class="help-steps">
          <li><span class="step-num">1</span><div><strong>选择固件文件</strong><p>点击“选择文件”，或直接在输入框中填写本地路径/在线 URL。</p></div></li>
          <li><span class="step-num">2</span><div><strong>获取分区列表</strong><p>点击“获取分区列表”解析 payload 内容并展示可提取分区。</p></div></li>
          <li><span class="step-num">3</span><div><strong>选择输出目录</strong><p>点击“更改”设置镜像提取后的输出目录。</p></div></li>
          <li><span class="step-num">4</span><div><strong>提取分区</strong><p>支持提取选中分区，也支持一键提取全部分区。</p></div></li>
        </ol>
      </div>
    </div>
  </el-dialog>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
});

const emit = defineEmits(['update:visible']);

const visibleProxy = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});
</script>

<style lang="scss" scoped>
.help-dialog {
  .el-dialog__header {
    padding: 20px 24px 14px;
    border-bottom: 1px solid var(--color-border);
  }

  .el-dialog__title {
    font-size: 15px;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .el-dialog__body {
    padding: 0;
  }
}

.help-content {
  padding: 4px 24px 24px;
  display: flex;
  flex-direction: column;
}

.help-section {
  padding: 16px 0;
  border-bottom: 1px solid var(--color-divider);
  &:last-child { border-bottom: none; }
}

.help-section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 10px;
}

.help-desc {
  font-size: 12.5px;
  color: var(--color-text-secondary);
  line-height: 1.8;
  margin: 0;
}

.help-steps {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;

  li {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }
}

.step-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--color-primary), var(--brand-info-strong));
  color: var(--text-on-primary);
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
  margin-top: 1px;
}
</style>
