<template>
  <div class="dl-content surface-card-strong">
    <div v-if="tasks.length === 0" class="empty-state">
      <div class="empty-pic">{{ activeTab === 'downloading' ? '📨' : '' }}</div>
      <div class="empty-text">{{ activeTab === 'downloading' ? '暂时没有下载任务' : '暂时没有传输完成的任务' }}</div>
      <el-button v-if="activeTab === 'downloading'" type="primary" plain size="small" @click="$emit('show-add')">点击新建</el-button>
    </div>

    <div v-else class="card-list">
      <div v-for="task in tasks" :key="task.id" class="dl-card-wrapper">
        <div class="dl-card" :class="{ finished: activeTab === 'completed' }">
          <div class="card-icon">
            <div class="icon-inner" :class="[getFileType(task.fileName), { finished: activeTab === 'completed' }]">
              <el-icon v-if="getFileType(task.fileName) === 'archive'"><ZipIcon /></el-icon>
              <el-icon v-else-if="getFileType(task.fileName) === 'video'"><VideoPlay /></el-icon>
              <el-icon v-else><Document /></el-icon>
            </div>
          </div>

          <div class="card-body">
            <div class="body-top">
              <span class="filename" :title="task.fileName">{{ task.fileName }}</span>
              <template v-if="activeTab === 'downloading'">
                <div class="speed-group">
                  <span class="speed-value">{{ task.speed }}</span>
                  <span v-if="task.eta && task.eta !== '--'" class="eta-value">剩余 {{ task.eta }}</span>
                </div>
              </template>
              <template v-else>
                <span class="time-text">{{ formatTime(task.completedAt) }}</span>
              </template>
            </div>

            <template v-if="activeTab === 'downloading'">
              <div class="body-mid">
                <el-progress :percentage="task.progress" :stroke-width="6" :show-text="false" :status="getProgressStatus(task.status)" class="main-progress" />
                <span class="progress-pct">{{ task.progress.toFixed(1) }}%</span>
              </div>
            </template>

            <div class="body-bottom">
              <div class="meta-left">
                <template v-if="activeTab === 'downloading'">
                  <span class="size-text">{{ task.downloaded }} <span class="divider">/</span> {{ task.totalSize }}</span>
                </template>
                <template v-else>
                  <span class="size-text">{{ task.totalSize }}</span>
                  <span class="save-path" :title="task.saveDir">{{ task.saveDir }}</span>
                </template>
                <el-tag size="small" :type="getStatusType(task.status)" effect="light" round class="status-tag">{{ getStatusLabel(task.status) }}</el-tag>
              </div>
              <div class="meta-right">
                <div class="action-btns" :class="{ 'show-always': activeTab === 'completed' }">
                  <template v-if="activeTab === 'downloading'">
                    <el-button link type="danger" @click="$emit('cancel-task', task)"><el-icon><Close /></el-icon>取消</el-button>
                  </template>
                  <template v-else>
                    <el-tooltip v-if="task.status === 'completed'" content="打开文件" placement="top">
                      <el-button link @click="$emit('open-file', task)"><el-icon><Document /></el-icon></el-button>
                    </el-tooltip>
                    <el-tooltip v-if="task.status === 'completed'" content="打开文件夹" placement="top">
                      <el-button link @click="$emit('open-folder', task)"><el-icon><FolderOpened /></el-icon></el-button>
                    </el-tooltip>
                    <el-tooltip v-if="['error', 'cancelled'].includes(task.status)" content="重试" placement="top">
                      <el-button link @click="$emit('retry-task', task)"><el-icon><Refresh /></el-icon></el-button>
                    </el-tooltip>
                    <el-tooltip content="清除记录" placement="top">
                      <el-button link type="danger" @click="$emit('remove-task', task)"><el-icon><Delete /></el-icon></el-button>
                    </el-tooltip>
                  </template>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Box as ZipIcon, Close, Delete, Document, FolderOpened, Refresh, VideoPlay } from '@element-plus/icons-vue';
defineProps({
  activeTab: { type: String, default: 'downloading' },
  tasks: { type: Array, default: () => [] },
  getFileType: { type: Function, required: true },
  getStatusLabel: { type: Function, required: true },
  getStatusType: { type: Function, required: true },
  getProgressStatus: { type: Function, required: true },
  formatTime: { type: Function, required: true },
});
defineEmits(['show-add', 'cancel-task', 'open-file', 'open-folder', 'retry-task', 'remove-task']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.dl-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}
.card-list { display: flex; flex-direction: column; gap: 12px; }

.dl-card-wrapper {
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 340ms);
}

.dl-card-wrapper:nth-child(1) {
  --page-enter-delay: 0ms;
}

.dl-card-wrapper:nth-child(2) {
  --page-enter-delay: 40ms;
}

.dl-card-wrapper:nth-child(3) {
  --page-enter-delay: 80ms;
}

.dl-card-wrapper:nth-child(4) {
  --page-enter-delay: 120ms;
}

.dl-card-wrapper:nth-child(5) {
  --page-enter-delay: 160ms;
}
.dl-card { background: var(--surface-elevated); border-radius: var(--radius-md); padding: 16px; border: 1px solid var(--color-divider); display: flex; gap: 16px; transition: all 0.3s ease; position: relative; overflow: hidden;
  &::before { content: ''; position: absolute; top: 0; left: 0; bottom: 0; width: 4px; background: transparent; transition: background 0.3s; }
  &:hover { transform: translateY(-2px); box-shadow: var(--shadow-card-hover); border-color: rgba(var(--color-primary-rgb), 0.18); background: var(--surface-elevated-strong); &::before { background: var(--color-primary); } .action-btns:not(.show-always) { opacity: 1; transform: translateX(0); } }
  &.finished { &::before { background: var(--color-success); } &:hover { border-color: rgba(var(--color-success-rgb), 0.28); } }
}
.card-icon .icon-inner {
  width: 52px; height: 52px; border-radius: 14px; display: flex; align-items: center; justify-content: center; font-size: 28px; background: var(--icon-info-gradient); color: var(--color-info);
  &.archive { background: var(--icon-violet-gradient); color: var(--brand-violet); }
  &.video { background: var(--icon-rose-gradient); color: var(--brand-rose); }
  &.finished { background: var(--icon-success-gradient); color: var(--color-success); }
}
.card-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 10px; }
.body-top { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.filename { font-size: 16px; font-weight: 600; color: var(--color-text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex: 1; }
.speed-group { display: flex; align-items: center; gap: 12px; flex-shrink: 0; }
.speed-value { color: var(--color-info); font-weight: 700; font-family: 'JetBrains Mono', 'Consolas', monospace; font-size: 14px; }
.eta-value, .time-text { color: var(--color-text-muted); font-size: 12px; }
.body-mid { display: flex; align-items: center; gap: 16px; .main-progress { flex: 1; :deep(.el-progress-bar__outer) { background-color: var(--surface-soft); } } }
.progress-pct { font-size: 13px; font-weight: 600; color: var(--color-text-secondary); min-width: 48px; text-align: right; }
.body-bottom { display: flex; align-items: center; justify-content: space-between; }
.meta-left { display: flex; align-items: center; gap: 12px; font-size: 13px; color: var(--color-text-secondary); }
.size-text { font-family: 'JetBrains Mono', 'Consolas', monospace; .divider { color: var(--color-text-muted); margin: 0 4px; } }
.save-path { font-size: 12px; color: var(--color-text-muted); max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; &::before { content: '📍 '; } }
.action-btns { display: flex; gap: 8px; opacity: 0; transform: translateX(10px); transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); &.show-always { opacity: 1; transform: none; } }
.empty-state { height: 400px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--color-text-muted); .empty-pic { font-size: 72px; margin-bottom: 20px; opacity: 0.5; } .empty-text { font-size: 15px; margin-bottom: 24px; } }
</style>
