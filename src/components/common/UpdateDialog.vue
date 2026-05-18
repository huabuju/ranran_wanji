<template>
  <GlassModal :show="showUpdateDialog" max-width="520px" container-class="app-dialog-shell">
    <!-- Header with Gradient and Icon -->
    <div class="update-header">
      <div class="icon-orbit">
        <el-icon class="rocket-icon"><Promotion /></el-icon>
      </div>
      <h2 class="title">发现新版本</h2>
      <div v-if="updateInfo.version" class="version-badges">
        <span class="badge current">{{ updateInfo.localVersion }}</span>
        <el-icon class="arrow-icon"><Right /></el-icon>
        <span class="badge latest">{{ updateInfo.version }}</span>
      </div>
      <div v-if="updateInfo.dateVersion || updateInfo.localDateVersion" class="date-version-row">
        <span>时间版本</span>
        <strong>{{ updateInfo.localDateVersion || '--' }}</strong>
        <el-icon class="date-arrow"><Right /></el-icon>
        <strong>{{ updateInfo.dateVersion || '--' }}</strong>
      </div>
    </div>

    <!-- Content Area -->
    <div class="update-content">
      <div class="section-title">
        <el-icon><Document /></el-icon>
        <span>更新日志 ({{ updateInfo.date }})</span>
      </div>
      <el-scrollbar max-height="240px" class="notes-scroll">
        <ul class="notes-list">
          <li v-for="(note, index) in updateInfo.notes" :key="index">
            <span class="dot"></span>
            <span class="text">{{ note }}</span>
          </li>
        </ul>
      </el-scrollbar>
    </div>

    <!-- Footer Buttons -->
    <div class="update-footer">
      <button class="btn btn-secondary" @click="close">稍后再说</button>
      <button class="btn btn-primary" @click="handleUpdate">
        <span>立即更新</span>
        <!-- <el-icon class="btn-icon"><Download /></el-icon> -->
      </button>
    </div>

    <!-- Decorative elements -->
    <div class="blob blob-1"></div>
    <div class="blob blob-2"></div>
  </GlassModal>
</template>

<script setup>
import { useUpdateStore } from '@/utils/updateStore';
import { Promotion, Right, Document, Download } from '@element-plus/icons-vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import GlassModal from './GlassModal.vue';

const { showUpdateDialog, updateInfo } = useUpdateStore();

function close() {
  showUpdateDialog.value = false;
}

async function handleUpdate() {
  // await openUrl(updateInfo.value.url || 'https://gitee.com/xiaowan12/toolkit-tauri-app');
  ElMessage.success('在线更新已移除，请前往群内下载更新！');
  showUpdateDialog.value = false;
}
</script>

<style lang="scss" scoped>
.update-header {
  padding: 40px 20px 30px;
  text-align: center;
  background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-hover) 100%);
  color: var(--text-on-primary);
  position: relative;

  .icon-orbit {
    width: 70px;
    height: 70px;
    background: var(--surface-overlay);
    border-radius: 22px;
    margin: 0 auto 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    transform: rotate(15deg);
    box-shadow: var(--shadow-card);
    
    .rocket-icon {
      font-size: 32px;
      transform: rotate(-15deg);
    }
  }

  .title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 12px;
    letter-spacing: 0.5px;
  }

  .version-badges {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;

    .badge {
      padding: 6px 14px;
      border-radius: 100px;
      font-size: 13px;
      font-family: 'Inter', monospace;
      font-weight: 600;
      line-height: 1;
      letter-spacing: 0.04em;
      border: 1px solid transparent;
      backdrop-filter: blur(10px);
    }

    .current {
      background: rgba(49, 46, 129, 0.3);
      color: rgba(255, 255, 255, 0.96);
      border-color: rgba(255, 255, 255, 0.22);
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
      // text-decoration: line-through;
      text-decoration-thickness: 1px;
      text-decoration-color: rgba(255, 255, 255, 0.45);
    }

    .latest {
      background: var(--surface-elevated-strong);
      color: var(--color-primary);
      font-weight: 700;
      border-color: rgba(255, 255, 255, 0.72);
      box-shadow: 0 8px 18px rgba(15, 23, 42, 0.12);
    }

    .arrow-icon {
      font-size: 14px;
      opacity: 0.92;
    }
  }

  .date-version-row {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin-top: 12px;
    padding: 7px 12px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.13);
    border: 1px solid rgba(255, 255, 255, 0.18);
    color: rgba(255, 255, 255, 0.78);
    font-size: 12px;
    line-height: 1;
    backdrop-filter: blur(10px);

    strong {
      color: rgba(255, 255, 255, 0.96);
      font-family: 'Inter', monospace;
      font-size: 12px;
      font-weight: 700;
      letter-spacing: 0.04em;
    }

    .date-arrow {
      font-size: 12px;
      opacity: 0.8;
    }
  }
}

.update-content {
  padding: 24px;

  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin-bottom: 12px;
    
    .el-icon {
      color: var(--color-primary);
    }
  }

  .notes-scroll {
    background: var(--surface-soft);
    border-radius: 12px;
    padding: 12px;
    border: 1px solid var(--color-border);
  }

  .notes-list {
    list-style: none;
    padding: 0;
    margin: 0;

    li {
      display: flex;
      align-items: flex-start;
      gap: 10px;
      margin-bottom: 8px;
      color: var(--color-text-secondary);
      font-size: 14px;
      line-height: 1.5;

      .dot {
        width: 6px;
        height: 6px;
        background: var(--color-primary);
        border-radius: 50%;
        margin-top: 7px;
        flex-shrink: 0;
      }
    }
  }
}

.update-footer {
  padding: 0 24px 24px;
  display: flex;
  gap: 12px;

  .btn {
    flex: 1;
    height: 48px;
    border-radius: 14px;
    border: none;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;

    &:active {
      transform: scale(0.96);
    }
  }

  .btn-secondary {
    background: var(--surface-soft);
    border: 1px solid var(--color-border);
    color: var(--color-text-secondary);
    &:hover {
      background: var(--surface-chip-hover);
    }
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-hover) 100%);
    color: var(--text-on-primary);
    box-shadow: 0 8px 20px rgba(var(--color-primary-rgb), 0.28);
    
    &:hover {
      box-shadow: 0 12px 25px rgba(var(--color-primary-rgb), 0.34);
      filter: brightness(1.05);
    }

    .btn-icon {
      font-size: 18px;
    }
  }
}

/* Decorations */
.blob {
  position: absolute;
  z-index: -1;
  filter: blur(40px);
  border-radius: 50%;
}
.blob-1 {
  width: 150px;
  height: 150px;
  background: rgba(var(--color-primary-rgb), 0.12);
  top: -50px;
  right: -50px;
}
.blob-2 {
  width: 120px;
  height: 120px;
  background: rgba(var(--color-primary-rgb), 0.08);
  bottom: -30px;
  left: -30px;
}

/* Animations */
@keyframes slide-up {
  from { opacity: 0; transform: translateY(40px); }
  to { opacity: 1; transform: translateY(0); }
}

.fade-scale-enter-active, .fade-scale-leave-active {
  transition: all 0.3s ease;
}
.fade-scale-enter-from, .fade-scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
