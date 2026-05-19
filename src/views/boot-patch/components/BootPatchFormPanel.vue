<template>
  <section class="boot-form-panel">
    <div :class="['workspace-column',{'is-magisk': !isKernelSuMode && !isApatchMode}]">
      <section class="workspace-stage workspace-stage--inputs">
        <div class="field-grid">
          <article class="field-block field-block--full" :class="{ 'is-ready': bootPath }">
            <div class="field-head">
              <div class="field-head__main">
                <div class="field-icon field-icon--primary">
                  <SmartIcon name="download" :size="16" color="var(--color-primary)" />
                </div>
                <div class="field-copy">
                  <div class="field-copy__title">
                    <span>Boot 文件 / 官方链接</span>
                    <el-tooltip
                      placement="bottom-start"
                      effect="light"
                      :offset="3"
                      :show-after="220"
                      popper-class="boot-source-warning-tooltip"
                    >
                      <template #content>
                        <div class="boot-source-warning-tooltip__content">
                          <p>在线链接仅支持官方完整卡刷包 (非ota包)</p>
                          <p>正常修补流程一般不会超过 5 分钟，如长时间卡住请重走流程</p>
                          <p>手动导入镜像前，请先确认当前机型实际要修补的是 <code>boot</code> 还是 <code>init_boot</code></p>
                          <p>如果不确定，建议直接使用在线链接，或先导入完整固件包让程序自动识别</p>
                          <p>未填入在线链接/选择任何镜像时，仍支持自动匹配ROM来修补</p>
                        </div>
                      </template>
                      <button type="button" class="field-warning-trigger" aria-label="Boot 来源注意事项">
                        <SmartIcon name="info" :size="14" color="var(--text-warning-strong)" :show-decoration="false" />
                      </button>
                    </el-tooltip>
                  </div>
                  <strong>支持镜像、Payload、固件包与在线 URL</strong>
                </div>
              </div>

              <div class="field-status-pill" :class="{ 'is-ready': bootPath }">
                {{ bootPath ? '已就绪' : '待填写' }}
              </div>
            </div>

            <div class="path-row path-row--primary">
              <el-input
                class="field-input"
                :model-value="bootPath"
                :placeholder="bootInputPlaceholder"
                clearable
                @update:model-value="$emit('update:boot-path', $event)"
              />

              <div class="field-actions">
                <el-button class="action-btn" type="primary" :disabled="isBusy" @click="$emit('select-boot')">
                  选择
                </el-button>
              </div>
            </div>

            <p class="field-tip">
              {{ bootInputTip }}
            </p>
          </article>

          <article class="field-block field-block--full" :class="{ 'is-ready': currentToolPath }">
            <div class="field-head">
              <div class="field-head__main">
                <div class="field-icon field-icon--info">
                  <SmartIcon name="package" :size="16" color="var(--color-info)" />
                </div>
                <div class="field-copy currentToolDescription">
                  <span>{{ currentToolTitle }}</span>
                </div>
              </div>

              <div class="field-status-pill" :class="{ 'is-ready': currentToolPath }">
                {{ currentToolPath ? '已选择' : '待选择' }}
              </div>
            </div>

            <div class="path-row">
              <el-select
                class="field-select tool-select"
                :model-value="currentToolPath"
                :disabled="isBusy"
                :placeholder="currentToolPlaceholder"
                @update:model-value="handleToolChange"
              >
                <el-option
                  v-for="option in currentToolOptions"
                  :key="option.value"
                  :label="option.label"
                  :value="option.value"
                />
              </el-select>
            </div>

            <!-- <div v-if="currentToolPath" class="field-preview">
              <span class="field-preview__label">当前版本</span>
              <span class="field-preview__text" :title="currentToolLabel">{{ currentToolLabel }}</span>
            </div> -->
          </article>

          <article
            class="field-block field-block--full field-block--collapsible"
            :class="{ 'is-ready': outputDir, 'is-collapsed': !isOutputExpanded }"
          >
            <div class="field-head">
              <div class="field-head__main">
                <div class="field-icon field-icon--success">
                  <SmartIcon name="folder" :size="16" color="var(--color-success)" />
                </div>
                <div class="field-copy">
                  <span>输出目录</span>
                  <!-- <strong>修补后的镜像与日志摘要将回传到这个目录</strong> -->
                </div>
              </div>

              <div class="field-head__aside">
                <!-- <div class="field-status-pill" :class="{ 'is-ready': outputDir }">
                  {{ outputDir ? '已指定' : '待指定' }}
                </div> -->
                <button
                  type="button"
                  class="field-toggle"
                  :aria-expanded="String(isOutputExpanded)"
                  @click="toggleOutputExpanded"
                >
                  <span>{{ isOutputExpanded ? '收起' : '展开' }}</span>
                  <SmartIcon
                    name="chevron_down"
                    :size="14"
                    color="var(--color-info)"
                    :show-background="false"
                    :show-decoration="false"
                  />
                </button>
              </div>
            </div>

            <div v-if="isOutputExpanded" class="field-collapse-body">
              <div class="path-row">
                <el-input class="field-input" :model-value="outputDir" readonly placeholder="请选择输出目录" />
                <el-button class="action-btn action-btn--soft" :disabled="isBusy" @click="$emit('select-output')">
                  选择
                </el-button>
              </div>

              <p class="field-tip">如果先选择了 Boot 文件，默认会自动带出同级目录作为输出位置。</p>
            </div>

            <!-- <div v-if="outputDir" class="field-preview">
              <span class="field-preview__label">输出位置</span>
              <span class="field-preview__text" :title="outputDir">{{ outputDir }}</span>
            </div> -->
          </article>

          <article v-if="isKernelSuMode" class="field-block field-block--full" :class="{ 'is-ready': kernelSuKmi }">
            <div class="field-head">
              <div class="field-head__main">
                <div class="field-icon field-icon--warning">
                  <SmartIcon name="system" :size="16" color="var(--text-warning-strong)" />
                </div>
                <div class="field-copy">
                  <span>Kernel KMI</span>
                  <strong>默认结合当前连接设备推断，也可按需手动指定</strong>
                </div>
              </div>

              <div class="field-status-pill" :class="{ 'is-ready': kernelSuKmi }">
                {{ kernelSuKmi ? '已设置' : '自动判断' }}
              </div>
            </div>

            <div class="path-row">
              <el-select
                class="field-select tool-select"
                :model-value="kernelSuKmi"
                default-first-option
                :reserve-keyword="false"
                :disabled="isBusy || !kernelSuPath"
                placeholder="请选择合适的版本，例如 android14-6.1"
                @update:model-value="$emit('update:kernel-su-kmi', $event || '')"
              >
                <el-option
                  v-for="option in kernelSuKmiOptions"
                  :key="option.value"
                  :label="option.label"
                  :value="option.value"
                />
              </el-select>
            </div>
            <!-- <div v-if="kernelSuDetectedKmi || kernelSuKmi" class="field-preview">
              <span class="field-preview__label">当前推断</span>
              <span class="field-preview__text" :title="kernelSuDetectedKmi || kernelSuKmi">
                {{ kernelSuDetectedKmi || kernelSuKmi }}
              </span>
            </div> -->
          </article>

          <article v-if="isApatchMode" class="field-block field-block--full" :class="{ 'is-ready': apatchSuperKey }">
            <div class="field-head">
              <div class="field-head__main">
                <div class="field-icon field-icon--warning">
                  <SmartIcon name="system" :size="16" color="var(--text-warning-strong)" />
                </div>
                <div class="field-copy">
                  <span>{{ currentPatchModeLabel }} SuperKey</span>
                  <strong>用于注入 APatch 类内核补丁，必须与后续管理器中使用的 SuperKey 保持一致</strong>
                </div>
              </div>

              <div class="field-status-pill" :class="{ 'is-ready': apatchSuperKey }">
                {{ apatchSuperKey ? '已设置' : '待设置' }}
              </div>
            </div>

            <div class="path-row">
              <el-input
                class="field-input"
                :model-value="apatchSuperKey"
                type="password"
                show-password
                clearable
                maxlength="63"
                :placeholder="`请输入 ${currentPatchModeLabel} SuperKey`"
                @update:model-value="$emit('update:apatch-super-key', $event)"
              />
            </div>

            <!-- <p class="field-tip">
              默认会自动生成一份符合官方要求的 24 位字母数字 SuperKey，你也可以手动修改；手动填写时需保持 8-63 位且只能包含字母和数字。当前实现会把完整明文 SuperKey 写入修补后文件名。
            </p> -->
          </article>

        </div>
      </section>

      <section class="workspace-stage action-stage">
        <div class="actions">
          <el-button
            class="run-btn"
            type="primary"
            :loading="patching"
            :disabled="isBusy || !canPatch"
            @click="$emit('start')"
          >
            仅修补
          </el-button>
          <el-button
            class="root-btn"
            :loading="rooting"
            :disabled="isBusy || !canRoot"
            @click="$emit('one-key-root')"
          >
            一键 Root
          </el-button>
          <el-button class="reset-btn" :disabled="isBusy" @click="$emit('reset')">重置</el-button>
        </div>
      </section>
    </div>
  </section>
</template>

<script setup>
import { computed, ref } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

function getPathTail(value) {
  const normalized = String(value || '').trim().replace(/[\\/]+$/, '');
  if (!normalized) return '';
  const parts = normalized.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || normalized;
}

function normalizePatchMode(value) {
  const normalized = String(value || '').trim().toLowerCase();
  if (normalized === 'kernelsu') return 'kernelsu';
  if (normalized === 'kernelsu_next') return 'kernelsu_next';
  if (normalized === 'sukisu_ultra') return 'sukisu_ultra';
  if (normalized === 'resukisu') return 'resukisu';
  if (normalized === 'magisk_alpha') return 'magisk_alpha';
  if (normalized === 'apatch') return 'apatch';
  if (normalized === 'folkpatch') return 'folkpatch';
  return 'magisk';
}

function getPatchModeLabel(value) {
  const normalized = normalizePatchMode(value);
  if (normalized === 'kernelsu') return 'KernelSU';
  if (normalized === 'kernelsu_next') return 'KernelSU_Next';
  if (normalized === 'sukisu_ultra') return 'SukiSU_Ultra';
  if (normalized === 'resukisu') return 'ReSukiSU';
  if (normalized === 'magisk_alpha') return 'Magisk_Alpha';
  if (normalized === 'apatch') return 'APatch';
  if (normalized === 'folkpatch') return 'FolkPatch';
  return 'Magisk';
}

const props = defineProps({
  modeState: {
    type: Object,
    default: () => ({}),
  },
  executionState: {
    type: Object,
    default: () => ({}),
  },
  sourceState: {
    type: Object,
    default: () => ({}),
  },
  toolState: {
    type: Object,
    default: () => ({}),
  },
  kernelSuState: {
    type: Object,
    default: () => ({}),
  },
  rootState: {
    type: Object,
    default: () => ({}),
  },
});

const modeState = computed(() => props.modeState || {});
const executionState = computed(() => props.executionState || {});
const sourceState = computed(() => props.sourceState || {});
const toolState = computed(() => props.toolState || {});
const kernelSuState = computed(() => props.kernelSuState || {});
const rootState = computed(() => props.rootState || {});
const patchMode = computed(() => modeState.value.patchMode || 'magisk');
const bootPath = computed(() => sourceState.value.bootPath || '');
const outputDir = computed(() => sourceState.value.outputDir || '');
const magiskApkPath = computed(() => toolState.value.magiskApkPath || '');
const magiskApkOptions = computed(() => toolState.value.magiskApkOptions || []);
const magiskApkDir = computed(() => toolState.value.magiskApkDir || '');
const apatchApkPath = computed(() => toolState.value.apatchApkPath || '');
const apatchApkOptions = computed(() => toolState.value.apatchApkOptions || []);
const apatchApkDir = computed(() => toolState.value.apatchApkDir || '');
const apatchSuperKey = computed(() => toolState.value.apatchSuperKey || '');
const kernelSuPath = computed(() => toolState.value.kernelSuPath || '');
const kernelSuOptions = computed(() => toolState.value.kernelSuOptions || []);
const kernelSuDir = computed(() => toolState.value.kernelSuDir || '');
const kernelSuKmi = computed(() => kernelSuState.value.kernelSuKmi || '');
const kernelSuKmiOptions = computed(() => kernelSuState.value.kernelSuKmiOptions || []);
const kernelSuDetectedKmi = computed(() => kernelSuState.value.kernelSuDetectedKmi || '');
const canPatch = computed(() => executionState.value.canPatch === true);
const canRoot = computed(() => executionState.value.canRoot === true);
const kernelSuRuntimeLoading = computed(() => executionState.value.kernelSuRuntimeLoading === true);
const patching = computed(() => executionState.value.patching === true);
const rooting = computed(() => executionState.value.rooting === true);
const rootTargetPartition = computed(() => rootState.value.rootTargetPartition || '');
const rootPatchToolLabel = computed(() => rootState.value.rootPatchToolLabel || '');

const emit = defineEmits([
  'select-boot',
  'select-output',
  'update:boot-path',
  'update:magisk-apk-path',
  'update:apatch-apk-path',
  'update:apatch-super-key',
  'update:kernel-su-path',
  'update:kernel-su-kmi',
  'start',
  'one-key-root',
  'reset',
]);

const isBusy = computed(() => patching.value || rooting.value || kernelSuRuntimeLoading.value);
const currentMode = computed(() => normalizePatchMode(patchMode.value));
const currentPatchModeLabel = computed(() => getPatchModeLabel(patchMode.value));
const isKernelSuMode = computed(() => ['kernelsu', 'kernelsu_next', 'sukisu_ultra', 'resukisu'].includes(currentMode.value));
const isApatchMode = computed(() => ['apatch', 'folkpatch'].includes(currentMode.value));
const bootInputPlaceholder = computed(() => (
  isApatchMode.value
    ? '请选择 boot.img / payload 文件/输入在线地址'
    : '请选择 boot.img / init_boot.img / payload 文件/输入在线地址'
));
const bootInputTip = computed(() => (
  isApatchMode.value
    ? `${currentPatchModeLabel.value} 模式已对齐 APatch 类方案要求：本地只支持 \`boot.img\`；如果导入 \`payload.bin\`、固件 \`zip\` 或在线 URL，程序只会提取并修补 \`boot\` 分区。`
    : '支持本地 `boot.img`、`init_boot.img`、`payload.bin`、固件 `zip`，也支持在线 URL。检测到 Payload 时会自动提取可修补分区。'
));
const currentToolOptions = computed(() => {
  if (isKernelSuMode.value) return kernelSuOptions.value;
  if (isApatchMode.value) return apatchApkOptions.value;
  return magiskApkOptions.value;
});
const currentToolPath = computed(() => {
  if (isKernelSuMode.value) return kernelSuPath.value;
  if (isApatchMode.value) return apatchApkPath.value;
  return magiskApkPath.value;
});
const currentToolLabel = computed(() => {
  const matched = currentToolOptions.value.find((item) => item.value === currentToolPath.value);
  return matched?.label || getPathTail(currentToolPath.value) || '未选择';
});
const currentToolTitle = computed(() => {
  return `${currentPatchModeLabel.value} 版本`;
});
const currentToolScanDir = computed(() => {
  if (isKernelSuMode.value) return String(kernelSuDir.value || '').trim();
  if (isApatchMode.value) return String(apatchApkDir.value || '').trim();
  return String(magiskApkDir.value || '').trim();
});

const currentToolPlaceholder = computed(() => {
  if (isKernelSuMode.value) return `请选择 ${currentPatchModeLabel.value} 版本`;
  if (isApatchMode.value) return `请选择 ${currentPatchModeLabel.value} APK`;
  return `请选择 ${currentPatchModeLabel.value} APK`;
});
const currentToolTip = computed(() => {
  if (isKernelSuMode.value) return `${currentPatchModeLabel.value} 模式会沿用 KernelSU 手机端修补流程，自动从 APK 中提取 ksud 与 magiskboot 后完成修补。`;
  if (isApatchMode.value) return `${currentPatchModeLabel.value} 模式会沿用 APatch 手机端修补流程，自动推送 APK 内的 kptools 与脚本后完成修补。`;
  if (currentMode.value === 'magisk_alpha') {
    return 'Magisk_Alpha 模式会沿用 Magisk 手机端修补流程，但仅使用 `bin/boot-patch/magisk_Alpha` 目录下的 APK 资源；请确保设备侧安装版本与当前所选版本保持一致。';
  }
  return 'Magisk 模式会沿用当前手机端修补流程，并使用 `bin/boot-patch/magisk` 目录下的 APK 资源；设备需要保持 ADB 正常连接。';
});
const effectiveRootToolLabel = computed(() => rootPatchToolLabel.value || currentToolLabel.value);
const effectiveRootTargetPartitionText = computed(() => (
  rootTargetPartition.value ? `，目标分区为 ${rootTargetPartition.value}` : ''
));
const isOutputExpanded = ref(false);

function toggleOutputExpanded() {
  isOutputExpanded.value = !isOutputExpanded.value;
}

function handleToolChange(value) {
  if (isKernelSuMode.value) {
    emit('update:kernel-su-path', value);
    return;
  }

  if (isApatchMode.value) {
    emit('update:apatch-apk-path', value);
    return;
  }

  emit('update:magisk-apk-path', value);
}
</script>

<style lang="scss" scoped>
.boot-form-panel {
  padding: 12px;
  box-sizing: border-box;
}

.workspace-column {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0 16px ;

  &.is-magisk{
      gap: 30px 16px ;
  }
}

.field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 18px;
}

.field-block {
  padding: 8px 14px;
  border-radius: 20px;
  background: var(--surface-panel);
  border: 1px solid var(--border-soft);
  transition: transform 0.24s ease, border-color 0.24s ease, box-shadow 0.24s ease, background 0.24s ease;
}

.field-block--full {
  grid-column: 1 / -1;
}

.field-block:hover {
  transform: translateY(-3px);
  border-color: rgba(var(--color-primary-rgb), 0.18);
  box-shadow: var(--shadow-sm);
}

.field-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.field-head__aside {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.field-head__main {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.field-icon {
  width: 42px;
  height: 42px;
  border-radius: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid transparent;
}

.field-icon--primary {
  background: rgba(var(--color-primary-rgb), 0.1);
  border-color: rgba(var(--color-primary-rgb), 0.12);
}

.field-icon--info {
  background: rgba(var(--color-info-rgb), 0.1);
  border-color: rgba(var(--color-info-rgb), 0.12);
}

.field-icon--success {
  background: rgba(var(--color-success-rgb), 0.1);
  border-color: rgba(var(--color-success-rgb), 0.12);
}

.field-icon--warning {
  background: rgba(var(--color-warning-rgb), 0.1);
  border-color: rgba(var(--color-warning-rgb), 0.14);
}

.field-copy {
  min-width: 0;
}

.field-copy__title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.field-copy__title > span {
  order: 1;
}

.field-copy span {
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 700;
}

.field-copy strong {
  display: flex;
  margin-top: 2px;
  width: 100%;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  line-height: 1.4;
  flex-wrap: wrap;
  word-break: break-all;
}

.field-warning-trigger {
  order: 2;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 1px solid rgba(var(--color-warning-rgb), 0.16);
  border-radius: 999px;
  background: rgba(var(--color-warning-rgb), 0.08);
  color: var(--text-warning-strong);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  cursor: help;
  transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease;
}

.field-warning-trigger:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--color-warning-rgb), 0.24);
  background: rgba(var(--color-warning-rgb), 0.14);
  box-shadow: 0 10px 20px -16px rgba(var(--color-warning-rgb), 0.8);
}

.field-warning-trigger:focus-visible {
  outline: none;
  border-color: rgba(var(--color-warning-rgb), 0.3);
  box-shadow: 0 0 0 3px rgba(var(--color-warning-rgb), 0.16);
}

.field-warning-trigger :deep(.smart-icon-container) {
  width: auto;
  height: auto;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.field-status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: 0 10px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(var(--color-warning-rgb), 0.22);
  background: rgba(var(--color-warning-rgb), 0.08);
  color: var(--text-warning-strong);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.field-status-pill.is-ready {
  border-color: rgba(var(--color-success-rgb), 0.18);
  background: rgba(var(--color-success-rgb), 0.08);
  color: var(--color-success);
}

.field-block--collapsible.is-collapsed {
  padding-bottom: 14px;
}

.field-block--collapsible.is-collapsed .field-head {
  margin-bottom: 0px;
}

.field-toggle {
  height: 28px;
  padding: 0 12px;
  border: 1px solid rgba(var(--color-info-rgb), 0.16);
  border-radius: var(--radius-full);
  background: rgba(var(--color-info-rgb), 0.08);
  color: var(--color-info);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease;
}

.field-toggle:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--color-info-rgb), 0.24);
  background: rgba(var(--color-info-rgb), 0.12);
}

.field-toggle:focus-visible {
  outline: none;
  border-color: rgba(var(--color-info-rgb), 0.28);
  box-shadow: 0 0 0 3px rgba(var(--color-info-rgb), 0.12);
}

.field-toggle span {
  font-size: 12px;
  font-weight: 700;
}

.field-toggle :deep(.smart-icon-svg) {
  transition: transform 0.2s ease;
  transform: rotate(0deg);
}

.field-toggle[aria-expanded='true'] :deep(.smart-icon-svg) {
  transform: rotate(180deg);
}

.path-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.path-row--primary {
  align-items: stretch;
}

.field-actions {
  display: flex;
  align-items: stretch;
  gap: 8px;
}

.field-tip {
  margin: 8px 0 0;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.7;
}

.field-collapse-body {
  display: grid;
  gap: 8px;
}

.field-preview {
  margin-top: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  background: var(--surface-soft);
  border: 1px dashed rgba(var(--color-primary-rgb), 0.16);
}

.field-preview__label {
  display: block;
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.field-preview__text {
  display: block;
  margin-top: 6px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tool-select {
  flex: 1;
}

.action-btn {
  min-width: 84px;
  height: 34px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(var(--color-primary-rgb), 0.18) !important;
  background: rgba(var(--color-primary-rgb), 0.08) !important;
  color: var(--color-primary) !important;
  box-shadow: none !important;
}

.action-btn:hover {
  transform: translateY(-1px);
  background: rgba(var(--color-primary-rgb), 0.12) !important;
  border-color: rgba(var(--color-primary-rgb), 0.24) !important;
}

.action-btn--soft {
  border-color: rgba(var(--color-info-rgb), 0.2) !important;
  background: rgba(var(--color-info-rgb), 0.08) !important;
  color: var(--color-info) !important;
}

.action-stage {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 10px;
  box-sizing: border-box;
  gap: 12px;
}

.actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

.run-btn,
.root-btn,
.reset-btn {
  height: 40px;
  padding: 12px 24px;
  box-sizing: border-box;
  border-radius: var(--radius-full);
}

.actions .run-btn {
  // min-width: 148px;
  border: none !important;
  background: linear-gradient(135deg, var(--color-primary), var(--brand-blue-strong)) !important;
  box-shadow: 0 14px 24px rgba(var(--color-primary-rgb), 0.24);
}

.actions .run-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 18px 28px rgba(var(--color-primary-rgb), 0.28);
}

.root-btn {
  // min-width: 148px;
  padding: 12px 24px;
  box-sizing: border-box;
  border-color: rgba(var(--color-success-rgb), 0.24) !important;
  background: linear-gradient(135deg, rgba(var(--color-success-rgb), 0.14), rgba(var(--color-info-rgb), 0.14)) !important;
  color: var(--color-success) !important;
}

.root-btn:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--color-success-rgb), 0.34) !important;
  box-shadow: 0 14px 24px rgba(var(--color-success-rgb), 0.16);
}

.reset-btn {
  border-color: rgba(var(--color-warning-rgb), 0.2) !important;
  background: rgba(var(--color-warning-rgb), 0.08) !important;
  color: var(--text-warning-strong) !important;
}

:deep(.field-input .el-input__wrapper),
:deep(.field-select .el-select__wrapper) {
  min-height: 38px;
  border-radius: 16px;
  background: var(--surface-elevated);
  box-shadow:
    inset 0 0 0 1px rgba(var(--color-primary-rgb), 0.08),
    0 10px 24px -18px rgba(15, 23, 42, 0.18) !important;
  transition: box-shadow 0.2s ease, background 0.2s ease;
}

:deep(.field-input .el-input__wrapper:hover),
:deep(.field-input .el-input__wrapper.is-focus),
:deep(.field-select .el-select__wrapper:hover),
:deep(.field-select .el-select__wrapper.is-focused) {
  background: var(--surface-elevated-strong);
  box-shadow:
    inset 0 0 0 1px rgba(var(--color-primary-rgb), 0.2),
    0 12px 28px -20px rgba(var(--color-primary-rgb), 0.32) !important;
}

:deep(.field-input .el-input__inner),
:deep(.field-select .el-select__selected-item) {
  font-size: 13px;
  color: var(--color-text-primary);
}

:deep(.field-input .el-input__inner::placeholder) {
  color: var(--color-text-muted);
}

:deep(.boot-source-warning-tooltip) {
  max-width: 340px;
  padding: 12px 14px;
  border-radius: 14px;
  border: 1px solid rgba(var(--color-warning-rgb), 0.18);
  background: linear-gradient(180deg, color-mix(in srgb, var(--warning-soft) 36%, var(--surface-elevated-strong)), var(--surface-elevated)) !important;
  box-shadow: var(--shadow-card);
}

:deep(.boot-source-warning-tooltip .boot-source-warning-tooltip__content) {
  display: grid;
  gap: 8px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

:deep(.boot-source-warning-tooltip .boot-source-warning-tooltip__content p) {
  margin: 0;
}

:deep(.boot-source-warning-tooltip code) {
  padding: 1px 5px;
  border-radius: 6px;
  background: rgba(var(--color-warning-rgb), 0.12);
  color: var(--text-warning-strong);
  font-size: 11px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

@media (max-width: 900px) {
  .field-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .boot-form-panel {
    padding: 12px;
  }

  .workspace-stage {
    padding: 12px;
  }

  .path-row {
    flex-direction: column;
    align-items: stretch;
  }

  .field-head {
    flex-direction: column;
  }

  .field-actions {
    width: 100%;
  }

  .field-actions :deep(.el-button),
  .path-row > :deep(.el-button) {
    flex: 1;
  }

  .actions {
    width: 100%;
    justify-content: stretch;
  }

  .actions :deep(.el-button) {
    flex: 1;
  }
}
</style>
