<template>
  <div ref="floatingLogRef" class="floating-log-wrapper">
    <transition name="slide-up">
      <div v-show="isOpen" class="log-drawer-panel">
        <div class="log-header">
          <div class="log-title">
            <SmartIcon name="terminal" :size="14" color="var(--color-primary)" />
            <span>执行日志</span>
          </div>
          <div class="log-actions">
            <el-button link type="primary" size="small" @click="$emit('clear')">清空</el-button>
            <div class="divider"></div>
            <el-button link type="primary" size="small" @click="handleCopy">复制</el-button>
            <div class="divider"></div>
          </div>
        </div>

        <div ref="logContainer" class="log-body">
          <template v-if="hasSummary">
            <div class="log-line summary-title">
              <span class="log-tag">[STATUS]</span>
              <span class="log-text">当前阶段：{{ currentStatusText }}</span>
            </div>

            <div v-if="outputPath" class="log-line summary-line">
              <span class="log-tag">[OUTPUT]</span>
              <span class="log-text">{{ outputPath }}</span>
            </div>

            <div v-if="packageZipPath" class="log-line summary-line">
              <span class="log-tag">[PACKAGE]</span>
              <span class="log-text">{{ packageZipPath }}</span>
            </div>

            <div v-if="imageMetaPreview" class="log-line summary-line">
              <span class="log-tag">[IMAGE]</span>
              <span class="log-text">{{ imageMetaPreview }}</span>
            </div>

            <div v-if="packageZipPath" class="summary-actions">
              <el-button size="small" type="primary" plain @click="$emit('open-package-dir')">
                打开资料包目录
              </el-button>
            </div>

            <div v-if="remoteWorkDir" class="log-line summary-line">
              <span class="log-tag">[REMOTE]</span>
              <span class="log-text">{{ remoteWorkDir }}</span>
            </div>

            <div
              v-for="(step, index) in steps"
              :key="step.key"
              class="log-line summary-step"
              :class="stepLogType(step.state)"
            >
              <span class="log-tag">[STEP {{ index + 1 }}]</span>
              <span class="log-text">{{ step.label }}｜{{ stateTextMap[step.state] || '待执行' }}｜{{ step.desc }}</span>
            </div>

            <div class="summary-divider"></div>
          </template>

          <div v-if="logs.length === 0" class="log-empty">等待执行操作...</div>
          <div
            v-for="(log, idx) in logs"
            :key="idx"
            class="log-line"
            :class="log.type"
          >
            <span v-if="log.time" class="log-time">[{{ log.time }}]</span>
            <span v-if="log.tag" class="log-tag">[{{ log.tag }}]</span>
            <span class="log-text">{{ log.content || log.msg }}</span>
          </div>
        </div>
      </div>
    </transition>

    <div
      class="float-btn"
      :class="{ 'has-new': hasNewLog, 'is-open': isOpen }"
      title="点击展开/收起日志"
      @click="toggleLog"
    >
      <SmartIcon
        name="terminal"
        class="terminal-icon"
        :size="24"
        :color="isOpen ? 'white' : 'var(--color-primary)'"
      />
      <div v-if="unreadCount > 0 && !isOpen" class="badge">
        {{ unreadCount > 99 ? '99+' : unreadCount }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  logs: {
    type: Array,
    default: () => [],
  },
  status: {
    type: String,
    default: '',
  },
  steps: {
    type: Array,
    default: () => [],
  },
  outputPath: {
    type: String,
    default: '',
  },
  packageZipPath: {
    type: String,
    default: '',
  },
  imageMetaPreview: {
    type: String,
    default: '',
  },
  remoteWorkDir: {
    type: String,
    default: '',
  },
});

const emit = defineEmits(['clear', 'open-package-dir']);

const isOpen = ref(false);
const unreadCount = ref(0);
const logContainer = ref(null);
const floatingLogRef = ref(null);
const hasNewLog = ref(false);
let newLogTimer = null;
let lastLogLen = 0;

const statusMeta = {
  idle: '等待开始',
  prepare: '准备参数',
  check: '校验依赖',
  push: '推送文件',
  patch: '执行修补',
  pull: '拉回结果',
  pack: '整理资料包',
  clean: '清理目录',
  done: '修补完成',
  error: '修补失败',
};

const stateTextMap = {
  idle: '待执行',
  active: '进行中',
  done: '已完成',
  error: '失败',
};

const hasSummary = computed(() => {
  return Boolean(
    props.status
    || props.steps.length
    || props.outputPath
    || props.packageZipPath
    || props.imageMetaPreview
    || props.remoteWorkDir,
  );
});

const currentStatusText = computed(() => statusMeta[props.status] || '等待开始');

const toggleLog = () => {
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    unreadCount.value = 0;
    hasNewLog.value = false;
    scrollToBottom();
  }
};

const openLog = () => {
  if (!isOpen.value) {
    isOpen.value = true;
  }
  unreadCount.value = 0;
  hasNewLog.value = false;
  scrollToBottom();
};

const closeLog = () => {
  isOpen.value = false;
};

watch(
  () => props.logs,
  (newVal) => {
    if (newVal.length > lastLogLen) {
      if (!isOpen.value) {
        unreadCount.value += newVal.length - lastLogLen;
        hasNewLog.value = true;
        if (newLogTimer) clearTimeout(newLogTimer);
        newLogTimer = setTimeout(() => {
          hasNewLog.value = false;
        }, 800);
      } else {
        scrollToBottom();
      }
    } else if (newVal.length === 0) {
      unreadCount.value = 0;
    }
    lastLogLen = newVal.length;
  },
  { deep: true },
);

const scrollToBottom = () => {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  });
};

const handleCopy = () => {
  const summaryLines = [];

  if (hasSummary.value) {
    summaryLines.push(`[STATUS] 当前阶段：${currentStatusText.value}`);
    if (props.outputPath) summaryLines.push(`[OUTPUT] ${props.outputPath}`);
    if (props.packageZipPath) summaryLines.push(`[PACKAGE] ${props.packageZipPath}`);
    if (props.imageMetaPreview) summaryLines.push(`[IMAGE] ${props.imageMetaPreview}`);
    if (props.remoteWorkDir) summaryLines.push(`[REMOTE] ${props.remoteWorkDir}`);
    props.steps.forEach((step, index) => {
      summaryLines.push(`[STEP ${index + 1}] ${step.label}｜${stateTextMap[step.state] || '待执行'}｜${step.desc}`);
    });
  }

  const logLines = (props.logs || []).map((log) => {
    const time = log.time ? `[${log.time}] ` : '';
    const tag = log.tag ? `[${log.tag}] ` : '';
    const text = log.content || log.msg || '';
    return `${time}${tag}${text}`;
  });

  const content = [...summaryLines, ...logLines].join('\n');

  if (!content) {
    ElMessage.warning('没有可复制的日志内容');
    return;
  }

  navigator.clipboard.writeText(content).then(() => {
    ElMessage.success('日志已复制到剪贴板');
  }).catch((err) => {
    console.error('Copy failed:', err);
    ElMessage.error('复制失败，请重试');
  });
};

const stepLogType = (state) => {
  if (state === 'done') return 'success';
  if (state === 'error') return 'error';
  if (state === 'active') return 'warning';
  return 'info';
};

const handleClickOutside = (event) => {
  if (isOpen.value && floatingLogRef.value && !floatingLogRef.value.contains(event.target)) {
    isOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  if (newLogTimer) clearTimeout(newLogTimer);
});

defineExpose({
  open: openLog,
  close: closeLog,
  toggle: toggleLog,
});
</script>

<style lang="scss" scoped>
.floating-log-wrapper {
  position: fixed;
  bottom: 86px;
  right: 24px;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  pointer-events: none;
}

.log-drawer-panel {
  pointer-events: auto;
  margin-bottom: 20px;
  width: 380px;
  height: calc(100vh - 330px);
  display: flex;
  flex-direction: column;
  background: var(--bg-glass);
  backdrop-filter: var(--blur-glass) saturate(180%);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-card-hover);
  overflow: hidden;
  transform-origin: bottom right;
}

.log-header {
  padding: 14px 18px;
  border-bottom: 1px solid var(--color-border);
  background: var(--surface-overlay);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.log-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.log-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.divider {
  width: 1px;
  height: 12px;
  background: var(--color-border);
}

.log-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
  background: var(--surface-panel);
  color: var(--text-code);
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
}

.log-empty {
  color: var(--color-text-muted);
  text-align: center;
  margin-top: 60px;
  font-style: italic;
}

.log-line {
  margin-bottom: 6px;
  word-wrap: break-word;
  word-break: break-all;
  white-space: pre-wrap;
  padding: 4px 6px;
  border-radius: 4px;
  user-select: text;
  -webkit-user-select: text;

  .log-time {
    color: var(--color-text-secondary);
    margin-right: 6px;
    font-size: 11px;
    user-select: text;
    -webkit-user-select: text;
  }

  .log-tag {
    color: var(--color-primary);
    margin-right: 6px;
    font-weight: 600;
    font-size: 11px;
    user-select: text;
    -webkit-user-select: text;
  }

  .log-text {
    user-select: text;
    -webkit-user-select: text;
  }

  &.success {
    background-color: rgba(var(--color-success-rgb), 0.1);
    color: var(--brand-success-strong);
  }

  &.error {
    background-color: rgba(var(--color-danger-rgb), 0.1);
    color: var(--color-danger);
  }

  &.warning {
    background-color: rgba(var(--color-warning-rgb), 0.1);
    color: var(--text-warning-strong);
  }

  &.info {
    color: var(--color-text-secondary);
  }

  &:hover {
    background-color: var(--surface-overlay);
  }
}

.summary-title {
  font-weight: 700;
}

.summary-divider {
  height: 1px;
  margin: 10px 0 12px;
  background: var(--color-border);
  opacity: 0.65;
}

.summary-actions {
  display: flex;
  justify-content: flex-end;
  margin: 10px 0 12px;
}

.float-btn {
  pointer-events: auto;
  position: relative;
  width: 54px;
  height: 54px;
  border-radius: 50%;
  background: var(--surface-elevated-strong);
  backdrop-filter: blur(8px);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-card);
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);

  &.is-open {
    background: var(--color-primary);
    border-color: var(--color-primary);
    transform: rotate(15deg) scale(0.95);
    box-shadow: 0 4px 15px rgba(var(--color-primary-rgb), 0.4);
  }

  &:hover:not(.is-open) {
    transform: scale(1.05) translateY(-2px);
    box-shadow: var(--shadow-card-hover);
  }

  &.has-new {
    animation: bounce-rotate 0.5s ease;
  }
}

.badge {
  position: absolute;
  top: 0;
  right: 0;
  transform: translate(25%, -25%);
  background: var(--color-danger);
  color: var(--text-on-primary);
  font-size: 11px;
  font-weight: 700;
  min-width: 18px;
  height: 18px;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
  border: 2px solid var(--surface-elevated-strong);
  box-shadow: var(--shadow-overlay-soft);
  animation: pop 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

@keyframes bounce-rotate {
  0% {
    transform: scale(1);
  }
  30% {
    transform: scale(1.1) rotate(-5deg);
  }
  50% {
    transform: scale(1.05) rotate(5deg);
  }
  70% {
    transform: scale(1.1) rotate(-3deg);
  }
  100% {
    transform: scale(1);
  }
}

@keyframes pop {
  0% {
    transform: translate(25%, -25%) scale(0);
  }
  100% {
    transform: translate(25%, -25%) scale(1);
  }
}
</style>
