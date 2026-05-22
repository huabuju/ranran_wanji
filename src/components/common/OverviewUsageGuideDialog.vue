<template>
  <el-dialog
    v-model="visible"
    title="工具使用说明"
    width="620px"
    align-center
    class="app-dialog-shell overview-usage-dialog"
    :close-on-click-modal="true"
    append-to-body
    destroy-on-close
  >
    <div class="guide-content">
      <div class="guide-hero">
        <div class="guide-hero-icon">
          <SmartIcon
            name="info"
            :size="18"
            color="var(--color-primary)"
            :show-background="false"
            :show-decoration="false"
          />
        </div>
        <div class="guide-hero-copy">
          <p class="guide-hero-title">首次使用前请先确认连接环境</p>
          <p class="guide-hero-desc">
            建议先通过数据线完成一次设备识别与授权，再进行无线调试配对！
          </p>
          <p class="guide-hero-desc">
            若你未正确连接设备，那么您将无法使用绝大多数功能！
          </p>
        </div>
      </div>

      <div class="guide-section">
        <div class="guide-section-title">前置条件</div>
        <ul class="guide-list">
          <li>手机已开启开发者选项（Developer Options）。</li>
          <li>已开启 USB 调试（USB Debugging），部分机型还需要额外开启“无线调试”。</li>
          <li>
            电脑已正确
            <button class="guide-link-button" type="button" @click="handleOpenDriverFolder">安装 ADB / Fastboot 驱动</button>
            ，设备管理器中不能出现异常驱动项。
          </li>
          <li>建议优先使用稳定的数据线和主板直连 USB 接口，避免识别不稳定。</li>
          <li>手机首次连接电脑时，需要在设备端同意 USB 调试授权提示。</li>
        </ul>
      </div>

      <div class="guide-section">
        <div class="guide-section-title">常见注意事项</div>
        <ul class="guide-list">
          <li>
            若无法识别设备，请优先检查数据线、驱动安装、USB 模式以及开发者选项是否被系统关闭。
          </li>
          <li>若配对或连接失败，可尝试关闭占用 ADB 的第三方手机助手、模拟器或命令行窗口后重试。</li>
          <li>部分品牌系统会限制后台网络发现或调试权限，必要时请重新打开无线调试并重新配对。</li>
          <li>如果更换了 USB 接口、电脑环境或重置了调试授权，可能需要重新授权或重新配对。</li>
        </ul>
      </div>
    </div>
  </el-dialog>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SmartIcon from '@/components/common/SmartIcon.vue';

const visible = ref(false);

function open() {
  visible.value = true;
}

async function handleOpenDriverFolder() {
  try {
    await invoke('open_driver_folder');
  } catch (error) {
    console.error('Failed to open driver folder:', error);
    ElMessage.error(`打开失败: ${error}`);
  }
}

defineExpose({ open });
</script>

<style lang="scss" scoped>
.overview-usage-dialog {
  .el-dialog__header {
    // padding: 20px 24px 10px;
  }

  .el-dialog__title {
    font-size: 16px;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .el-dialog__body {
    padding: 0 24px 24px;
  }
}

.guide-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.guide-hero {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 16px 18px;
  border-radius: 16px;
  background: linear-gradient(135deg, rgba(var(--color-primary-rgb), 0.12), rgba(var(--color-info-rgb), 0.08));
  border: 1px solid rgba(var(--color-primary-rgb), 0.14);
}

.guide-hero-icon {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--color-primary-rgb), 0.1);
  box-shadow: inset 0 0 0 1px rgba(var(--color-primary-rgb), 0.08);
  flex-shrink: 0;
}

.guide-hero-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.guide-hero-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.4;
}

.guide-hero-desc {
  font-size: 12.5px;
  line-height: 1.8;
  color: var(--color-text-secondary);
}

.guide-section {
  padding: 4px 2px 0;
}

.guide-section-title {
  margin-bottom: 10px;
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.guide-list {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;

  li {
    font-size: 12.5px;
    line-height: 1.75;
    color: var(--color-text-secondary);
  }
}

.guide-link-button {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-primary);
  font: inherit;
  cursor: pointer;

  &:hover,
  &:focus-visible {
    text-decoration: underline;
  }
}
</style>
