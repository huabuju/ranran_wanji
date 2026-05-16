<template>
  <div class="bootstrap-shell">
    <div class="bootstrap-card">
      <section class="bootstrap-brand">
        <div class="bootstrap-brand__inner">
          <div class="bootstrap-badge">Runtime Bootstrap</div>
          <h1 class="bootstrap-title">然然玩机工具箱</h1>
          <p class="bootstrap-subtitle">
            启动前自动检查并准备 ADB、Fastboot、scrcpy 等核心工具依赖，让设备连接、调试与投屏能力随时可用。
          </p>

          <div class="bootstrap-highlights">
            <article
              v-for="item in bootstrapHighlights"
              :key="item.title"
              class="bootstrap-highlight"
            >
              <span class="bootstrap-highlight__label">{{ item.label }}</span>
              <strong class="bootstrap-highlight__title">{{ item.title }}</strong>
              <p class="bootstrap-highlight__text">{{ item.text }}</p>
            </article>
          </div>

          <div class="bootstrap-brand__footer">
            <span class="bootstrap-brand__line"></span>
            <p>Professional Android device toolkit with automated runtime preparation.</p>
          </div>
        </div>
      </section>

      <section class="bootstrap-panel">
        <div class="bootstrap-content">
          <p class="bootstrap-panel__eyebrow">System Initialization</p>
          <h2 class="bootstrap-panel__title">正在准备运行环境</h2>
          <p class="bootstrap-text">{{ message }}</p>

          <div class="bootstrap-progress">
            <div class="bootstrap-progress__bar" :style="{ width: `${progress}%` }"></div>
          </div>

          <div class="bootstrap-meta">
            <span>{{ roundedProgress }}%</span>
            <span v-if="runtimePhaseLabel">{{ runtimePhaseLabel }}</span>
          </div>

          <div class="bootstrap-stats">
            <div class="bootstrap-stat">
              <span class="bootstrap-stat__label">Status</span>
              <strong class="bootstrap-stat__value">{{ runtimeStatusText }}</strong>
            </div>
            <div class="bootstrap-stat">
              <span class="bootstrap-stat__label">Phase</span>
              <strong class="bootstrap-stat__value">{{ runtimePhaseLabel || '等待开始' }}</strong>
            </div>
            <div class="bootstrap-stat">
              <span class="bootstrap-stat__label">Progress</span>
              <strong class="bootstrap-stat__value">{{ roundedProgress }}%</strong>
            </div>
          </div>

          <div class="bootstrap-steps">
            <div
              v-for="(step, index) in runtimeSteps"
              :key="step.key"
              class="bootstrap-step"
              :class="{
                'is-done': !error && index < runtimeStepIndex,
                'is-active': !error && index === runtimeStepIndex,
                'is-error': !!error && index === runtimeStepIndex,
              }"
            >
              <span class="bootstrap-step__index">{{ index + 1 }}</span>
              <div class="bootstrap-step__body">
                <strong class="bootstrap-step__title">{{ step.title }}</strong>
                <span class="bootstrap-step__text">{{ step.text }}</span>
              </div>
            </div>
          </div>


          <div class="bootstrap-actions">
            <el-button type="primary" size="large" :loading="preparing" @click="$emit('retry')">
              {{ error ? '重试初始化' : '正在初始化' }}
            </el-button>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  message: {
    type: String,
    default: '',
  },
  progress: {
    type: Number,
    default: 0,
  },
  error: {
    type: String,
    default: '',
  },
  preparing: {
    type: Boolean,
    default: false,
  },
  phase: {
    type: String,
    default: 'check',
  },
});

defineEmits(['retry']);

const bootstrapHighlights = [
  {
    label: 'Device Bridge',
    title: 'ADB / Fastboot',
    text: '覆盖设备识别、调试连接、重启模式切换等常用底层能力。',
  },
  {
    label: 'Screen Control',
    title: 'scrcpy Runtime',
    text: '提供稳定的投屏与交互链路，减少首次连接时的等待时间。',
  },
  {
    label: 'Dependency Sync',
    title: 'On-Demand Assets',
    text: '按需校验并安装运行时依赖，减轻发版体积并降低缺失风险。',
  },
  {
    label: 'Wireless Debug',
    title: 'ADB Pairing',
    text: '支持无线调试配对与连接，方便多场景下快速接入设备。',
  },
  {
    label: 'Package Manager',
    title: 'App Control',
    text: '支持应用安装、提取、禁用与包信息查看等常用管理操作。',
  },
  {
    label: 'ROM Resources',
    title: 'Download Hub',
    text: '整合机型资源、系统包与相关工具下载，提升刷机准备效率。',
  },
];

const runtimeSteps = [
  { key: 'check', title: '检查环境', text: '检测本地是否已存在可用依赖。', phases: ['check'] },
  {
    key: 'fetch',
    title: '获取资源',
    text: '读取清单并下载、合并所需运行时资源。',
    phases: ['manifest', 'download', 'combine'],
  },
  {
    key: 'install',
    title: '校验安装',
    text: '校验文件完整性并解压安装到运行目录。',
    phases: ['verify', 'extract'],
  },
  { key: 'ready', title: '准备完成', text: '预热工具链并进入主界面。', phases: ['ready'] },
];

const runtimePhaseLabel = computed(() => {
  const labels = {
    check: '检查环境',
    manifest: '读取清单',
    download: '下载分卷',
    combine: '合并分卷',
    verify: '校验文件',
    extract: '解压依赖',
    ready: '准备完成',
  };

  return labels[props.phase] || '';
});

const runtimeStepIndex = computed(() => {
  const index = runtimeSteps.findIndex((step) => step.phases.includes(props.phase));
  return index >= 0 ? index : 0;
});

const runtimeStatusText = computed(() => {
  if (props.error) {
    return '等待重试';
  }

  if (props.phase === 'ready') {
    return '准备完成';
  }

  return '进行中';
});

const roundedProgress = computed(() => Math.round(props.progress));
</script>

<style lang="scss" scoped>
@use '@/views/overview/components/shared.scss' as overview;

.bootstrap-shell {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: stretch;
  justify-content: stretch;
  overflow: auto;
  padding: 0;
  background:
    radial-gradient(circle at top left, rgba(var(--color-info-rgb), 0.18), transparent 30%),
    radial-gradient(circle at 85% 18%, var(--brand-blue-soft), transparent 24%),
    radial-gradient(circle at bottom right, var(--brand-orange-soft), transparent 28%),
    linear-gradient(135deg, var(--bg-app) 0%, var(--surface-panel) 46%, var(--bg-app) 100%);
}

.bootstrap-card {
  flex: 1;
  display: grid;
  grid-template-columns: minmax(0, 55fr) minmax(360px, 35fr);
  gap: clamp(16px, 1.2vw, 24px);
  align-items: start;
  padding: 30px 32px 28px;
}

.bootstrap-brand {
  position: relative;
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
}

.bootstrap-brand__inner {
  width: 100%;
  position: relative;
  padding: 10px 0 0;
}

.bootstrap-brand__inner::before {
  content: '';
  position: absolute;
  inset: -36px -28px auto auto;
  width: 220px;
  height: 220px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(14, 165, 233, 0.14) 0%, rgba(14, 165, 233, 0) 72%);
  pointer-events: none;
  filter: blur(4px);
}

.bootstrap-badge {
  display: inline-flex;
  padding: 7px 14px;
  border-radius: 999px;
  background: var(--info-soft);
  color: var(--color-info);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.bootstrap-title {
  margin: 14px 0 16px;
  font-size: 22px;
  line-height: 1.12;
  letter-spacing: -0.03em;
  color: var(--color-text-primary);
}

.bootstrap-subtitle {
  max-width: 700px;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 18px;
  line-height: 1.9;
}

.bootstrap-highlights {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  margin-top: 34px;
}

.bootstrap-highlight {
  min-width: 0;
  padding: 18px 18px 16px;
  border-radius: 22px;
  background: linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
  border: 1px solid var(--border-soft);
  box-shadow:
    inset 0 1px 0 var(--border-strong),
    var(--shadow-card);
  backdrop-filter: blur(18px);
  transition: transform 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
}

.bootstrap-highlight:hover {
  @include overview.overview-hover-lift(-2px, var(--color-primary), var(--shadow-card));
}

.bootstrap-highlight__label {
  display: inline-flex;
  margin-bottom: 12px;
  color: var(--color-info);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.bootstrap-highlight__title {
  display: block;
  color: var(--color-text-primary);
  font-size: 16px;
  line-height: 1.4;
}

.bootstrap-highlight__text {
  margin: 10px 0 0;
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.8;
}

.bootstrap-brand__footer {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 28px;
  color: var(--color-text-muted);
  font-size: 14px;
  opacity: 0.9;
}

.bootstrap-brand__footer p {
  margin: 0;
}

.bootstrap-brand__line {
  width: 72px;
  height: 1px;
  background: linear-gradient(90deg, rgba(14, 165, 233, 0.9), rgba(59, 130, 246, 0.1));
}

.bootstrap-panel {
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
  min-width: 0;
  padding-top: 0;
}

.bootstrap-content {
  width: 100%;
  padding: 20px 34px 32px;
  border-radius: 20px;
  box-sizing: border-box;
  background: linear-gradient(180deg, var(--surface-elevated-strong), var(--surface-elevated));
  border: 1px solid var(--border-soft);
  box-shadow:
    inset 0 1px 0 var(--border-strong),
    var(--shadow-card-hover);
  backdrop-filter: blur(26px);
}

.bootstrap-panel__eyebrow {
  margin: 0;
  color: var(--color-info);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.bootstrap-panel__title {
  margin: 16px 0;
  color: var(--color-text-primary);
  font-size: 22px;
  line-height: 1.22;
}

.bootstrap-text {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 15px;
  line-height: 1.8;
}

.bootstrap-progress {
  margin-top: 26px;
  height: 12px;
  border-radius: 999px;
  background: var(--surface-chip);
  overflow: hidden;
}

.bootstrap-progress__bar {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--color-info) 0%, var(--brand-blue-strong) 54%, var(--brand-orange) 100%);
  transition: width 0.25s ease;
}

.bootstrap-meta {
  margin-top: 14px;
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 14px;
  color: var(--color-text-muted);
}

.bootstrap-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
  margin-top: 16px;
}

.bootstrap-stat {
  padding: 8px 10px 9px;
  border-radius: 12px;
  background: linear-gradient(180deg, var(--surface-panel-strong), var(--surface-panel));
  border: 1px solid var(--border-soft);
  box-shadow:
    inset 0 1px 0 var(--border-strong),
    var(--shadow-sm);
}

.bootstrap-stat__label {
  display: block;
  color: var(--color-text-muted);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.bootstrap-stat__value {
  display: block;
  margin-top: 6px;
  color: var(--color-text-primary);
  font-size: 12px;
  line-height: 1.25;
  font-weight: 700;
}

.bootstrap-steps {
  display: grid;
  gap: 12px;
  margin-top: 24px;
}

.bootstrap-step {
  display: flex;
  gap: 14px;
  align-items: flex-start;
  padding: 14px 16px;
  border-radius: 20px;
  background: var(--surface-panel);
  border: 1px solid var(--border-soft);
  box-shadow: inset 0 1px 0 var(--border-strong);
  transition:
    transform 0.2s ease,
    border-color 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;
}

.bootstrap-step.is-active {
  background: var(--info-soft);
  border-color: rgba(var(--color-info-rgb), 0.32);
  box-shadow: 0 14px 32px rgba(var(--color-info-rgb), 0.12);
  transform: translateY(-1px);
}

.bootstrap-step.is-done {
  background: var(--success-soft);
  border-color: rgba(var(--color-success-rgb), 0.3);
}

.bootstrap-step.is-error {
  background: var(--danger-soft);
  border-color: rgba(var(--color-danger-rgb), 0.3);
}

.bootstrap-step__index {
  flex: 0 0 auto;
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--surface-chip);
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 700;
}

.bootstrap-step.is-active .bootstrap-step__index {
  background: linear-gradient(135deg, var(--color-info), var(--brand-blue-strong));
  color: var(--text-on-primary);
}

.bootstrap-step.is-done .bootstrap-step__index {
  background: linear-gradient(135deg, var(--color-success), var(--brand-success-strong));
  color: var(--text-on-primary);
}

.bootstrap-step.is-error .bootstrap-step__index {
  background: linear-gradient(135deg, var(--color-danger), var(--brand-rose));
  color: var(--text-on-primary);
}

.bootstrap-step__body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bootstrap-step__title {
  color: var(--color-text-primary);
  font-size: 14px;
  line-height: 1.35;
}

.bootstrap-step__text {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.bootstrap-actions {
  margin-top: 32px;
  display: flex;
  justify-content: center;
}

@media (max-width: 1180px) {
  .bootstrap-card {
    grid-template-columns: 1fr;
    gap: 20px;
    padding: 24px;
  }

  .bootstrap-brand {
    padding: 16px 8px 0;
  }

  .bootstrap-brand__inner,
  .bootstrap-content {
    width: 100%;
  }
}

@media (max-width: 900px) {
  .bootstrap-content {
    padding: 32px 24px;
    border-radius: 24px;
  }

  .bootstrap-title {
    font-size: 34px;
  }

  .bootstrap-subtitle {
    font-size: 15px;
  }

  .bootstrap-highlights {
    grid-template-columns: 1fr;
  }

  .bootstrap-panel__title {
    font-size: 24px;
  }

  .bootstrap-stats {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .bootstrap-card {
    padding: 18px;
  }

  .bootstrap-brand {
    padding: 8px 0 0;
  }

  .bootstrap-highlights {
    grid-template-columns: 1fr;
  }

  .bootstrap-content {
    padding: 24px 18px;
  }

  .bootstrap-title {
    font-size: 30px;
  }

  .bootstrap-meta {
    font-size: 13px;
  }
}
</style>
