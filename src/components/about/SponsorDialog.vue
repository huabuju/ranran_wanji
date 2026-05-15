<template>
  <el-dialog
    v-model="visible"
    title=""
    width="560px"
    :show-close="false"
    align-center
    class="app-dialog-shell sponsor-dialog"
  >
    <div class="sponsor-content">
      <!-- 顶部咖啡图标 -->
      <div class="icon-header">
        <div class="cafe-icon-wrapper">
          <SmartIcon name="package" :size="32" color="var(--brand-pink)" />
        </div>
      </div>

      <!-- 文字标题与描述 -->
      <h2 class="title">赞助支持</h2>
      <p class="description">
        如果您觉得本工具对您有帮助，欢迎扫码请作者喝杯咖啡！您的支持是我持续维护项目的最大动力。
      </p>

      <!-- 二维码网格 -->
      <div class="qr-grid">
        <div class="qr-item">
          <div class="qr-card">
            <img src="@/assets/sponsor/alipay.jpg" alt="支付宝" />
          </div>
          <span class="qr-label">支付宝</span>
        </div>
        <div class="qr-item">
          <div class="qr-card">
            <img src="@/assets/sponsor/wechat.png" alt="微信支付" />
          </div>
          <span class="qr-label">微信支付</span>
        </div>
        <div class="qr-item">
          <div class="qr-card-alt">
            <img src="@/assets/sponsor/coffee.png" alt="Buy Me a Coffee" />
            <div class="inner-icon">
               <SmartIcon name="package" :size="16" color="var(--brand-yellow)" />
            </div>
          </div>
          <span class="qr-label">Buy Me a Coffee</span>
        </div>
      </div>

      <!-- 底部关闭按钮 -->
      <div class="footer-actions">
        <el-button class="close-btn" @click="visible = false">关闭</el-button>
      </div>
    </div>
  </el-dialog>
</template>

<script setup>
import { ref } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const visible = ref(false);

const open = () => {
  visible.value = true;
};

defineExpose({ open });
</script>

<style lang="scss">
.sponsor-dialog {
  --app-dialog-bg: var(--bg-glass);
  --app-dialog-border: var(--color-border);
  --app-dialog-radius: 28px;
  
  .el-dialog__header {
    display: none;
  }

  .el-dialog__body {
    padding: 40px !important;
  }
}

.sponsor-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;

  .icon-header {
    margin-bottom: 20px;
    
    .cafe-icon-wrapper {
      width: 64px;
      height: 64px;
      background: var(--brand-pink-soft);
      border-radius: 18px;
      display: flex;
      align-items: center;
      justify-content: center;
      box-shadow: var(--brand-pink-shadow-soft);
    }
  }

  .title {
    font-size: 22px;
    font-weight: 700;
    color: var(--color-text-primary);
    margin-bottom: 12px;
  }

  .description {
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.6;
    max-width: 400px;
    margin-bottom: 32px;
  }

  .qr-grid {
    display: flex;
    gap: 20px;
    margin-bottom: 40px;
    width: 100%;
    justify-content: center;

    .qr-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 12px;
      flex: 1;

      .qr-card, .qr-card-alt {
        width: 140px;
        height: 140px;
        background: var(--surface-panel-strong);
        border-radius: 16px;
        padding: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--color-border);
        position: relative;
        
        img {
          width: 100%;
          height: 100%;
          border-radius: 10px;
          object-fit: cover;
        }
      }

      .qr-card-alt {
        background: linear-gradient(180deg, var(--brand-yellow-soft), var(--surface-panel-strong));
        .inner-icon {
          position: absolute;
          background: var(--surface-elevated-strong);
          padding: 4px;
          border-radius: 50%;
          bottom: -4px;
          right: -4px;
          box-shadow: var(--shadow-overlay-soft);
        }
      }

      .qr-label {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-text-secondary);
      }
    }
  }

  .footer-actions {
    width: 100%;
    .close-btn {
      width: 120px;
      height: 44px;
      border-radius: 12px;
      background: var(--surface-soft);
      border: none;
      font-weight: 600;
      color: var(--color-text-primary);
      transition: all 0.2s;

      &:hover {
        background: var(--surface-chip-hover);
        transform: translateY(-1px);
      }
    }
  }
}
</style>
