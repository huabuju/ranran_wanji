<template>
  <section class="boot-hero-panel">
    <div class="hero-subbar page-subtoolbar surface-card">
      <div class="page-filter-left">
        <div class="page-chip-group">
          <div
            v-for="item in readinessChips"
            :key="item.key"
            class="page-chip hero-chip"
            :class="{ 'is-active': item.ready }"
          >
            <SmartIcon
              :name="item.icon"
              :size="12"
              :color="item.ready ? 'var(--color-primary)' : 'var(--color-text-muted)'"
              :show-background="false"
              :show-decoration="false"
            />
            <span>{{ item.label }}</span>
          </div>
        </div>
      </div>

      <div class="page-filter-right hero-subbar__tips">

        <el-dropdown trigger="click" @command="handlePatchModeCommand" placement="bottom-end" popper-class="boot-patch-mode-dropdown">
          <button type="button" class="hero-tip-chip hero-tip-chip--action">
            <SmartIcon name="system" :size="12" color="var(--color-primary)" :show-background="false" :show-decoration="false" />
            <span>修补方案：{{ currentPatchModeLabel }}</span>
            <svg class="dropdown-caret" viewBox="0 0 1024 1024" width="10" height="10" fill="currentColor">
              <path d="M512 640 256 384h512z" />
            </svg>
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="item in patchModeOptions"
                :key="item.value"
                :command="item.value"
                :class="{ 'is-selected': item.value === patchMode }"
              >
                {{ item.label }}
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>

        <button type="button" class="hero-tip-chip hero-tip-chip--action" @click="usageDialogVisible = true">
          <SmartIcon name="info" :size="12" color="var(--color-primary)" :show-background="false" :show-decoration="false" />
          <span>使用说明</span>
        </button>
      </div>
    </div>

    <div class="hero-grid">
      <div
        class="hero-main surface-card-strong"
        v-loading="kernelSuRuntimeLoading"
        element-loading-text="正在计算可选 KMI，请稍候..."
        element-loading-background="var(--surface-overlay)"
      >
        <div class="hero-console">
          <div class="hero-form-shell">
            <BootPatchFormPanel
              :mode-state="modeState"
              :execution-state="executionState"
              :source-state="sourceState"
              :tool-state="toolState"
              :kernel-su-state="kernelSuState"
              :root-state="rootState"
              @select-boot="$emit('select-boot')"
              @update:boot-path="$emit('update:boot-path', $event)"
              @update:magisk-apk-path="$emit('update:magisk-apk-path', $event)"
              @update:apatch-apk-path="$emit('update:apatch-apk-path', $event)"
              @update:apatch-super-key="$emit('update:apatch-super-key', $event)"
              @update:kernel-su-path="$emit('update:kernel-su-path', $event)"
              @update:kernel-su-kmi="$emit('update:kernel-su-kmi', $event)"
              @select-output="$emit('select-output')"
              @start="$emit('start')"
              @one-key-root="$emit('one-key-root')"
              @reset="$emit('reset')"
            />
          </div>
        </div>
      </div>

      <aside class="hero-aside surface-card">
        <div class="hero-aside-card">
          <span class="hero-aside-label">PATCH PROFILE</span>

          <div class="hero-risk-card">
            <span class="hero-risk-card__label">风险提示</span>
            <p>{{ riskNotice }}</p>
          </div>

          <div class="hero-switch-section">
            <span class="hero-switch-section__title">修补开关</span>
            <p class="hero-switch-section__desc">{{ switchSectionDescription }}</p>

            <div v-if="patchOptionChecklist.length" class="hero-checklist">
              <article
                v-for="item in patchOptionChecklist"
                :key="item.key"
                class="check-card"
                :class="{ 'is-checked': item.checked, 'is-disabled': patching || rooting }"
                role="button"
                tabindex="0"
                @click="toggleOption(item)"
                @keydown.enter.prevent="toggleOption(item)"
                @keydown.space.prevent="toggleOption(item)"
              >
                <div class="check-card__content">
                  <span>{{ item.label }}</span>
                  <strong>{{ item.description }}</strong>
                </div>

                <div class="check-card__mark" :class="{ 'is-checked': item.checked }">
                  <span class="check-card__tick"></span>
                </div>
              </article>
            </div>
          </div>
        </div>
      </aside>
    </div>

    <el-dialog
      v-model="usageDialogVisible"
      title="修补使用说明"
      width="620px"
      append-to-body
      class="app-dialog-shell boot-usage-dialog"
    >
      <div class="usage-dialog">
        <div class="usage-note">
          <p>
            1、如需要扩展可选修补版本，可前往
            <RouterLink class="usage-link" to="/github-apk" @click="usageDialogVisible = false">GitHub APK</RouterLink>
            页面/其他途径下载对应 Root 管理软件，并放入对应资源目录下
          </p>
          <p>
            2、应用会自动识别对应
            <button
              type="button"
              class="usage-link"
              @click="handleOpenBootPatchFolder"
            >
              目录下的资源包
            </button>
            ；放置完成后重启工具箱即可看到新增版本
          </p>
          <p>3、请确保修补版本与手机安装的管理器版本相一致（一键 Root 时请先卸载管理器，避免自动安装失败），如手动选择本地boot等镜像，需要确保文件名即为标准分区名（不带插槽名字）</p>
          <p>4、常用 Boot 管理器项目地址：</p>
          <div class="manager-link-list">
            <template v-for="(item, index) in bootManagerLinks" :key="item.name">
              <button
                type="button"
                class="usage-link manager-link"
                @click="openExternalLink(item.url)"
              >
                {{ item.name }}
              </button><span v-if="index < bootManagerLinks.length - 1" class="manager-link-separator">、</span>
            </template>
          </div>
        </div>
      </div>
    </el-dialog>
  </section>
</template>

<script setup>
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import SmartIcon from '@/components/common/SmartIcon.vue';
import BootPatchFormPanel from './BootPatchFormPanel.vue';

const usageDialogVisible = ref(false);

const bootManagerLinks = [
  { name: 'Magisk', url: 'https://github.com/topjohnwu/Magisk' },
  { name: 'Magisk-alpha', url: 'https://github.com/vvb2060/Magisk' },
  { name: 'KernelSU', url: 'https://github.com/tiann/KernelSU' },
  { name: 'KernelSU-Next', url: 'https://github.com/KernelSU-Next/KernelSU-Next' },
  { name: 'SukiSU-Ultra', url: 'https://github.com/SukiSU-Ultra/SukiSU-Ultra' },
  { name: 'ReSukiSU', url: 'https://github.com/ReSukiSU/ReSukiSU' },
  { name: 'APatch', url: 'https://github.com/bmax121/APatch/' },
  { name: 'FolkPatch', url: 'https://github.com/LyraVoid/FolkPatch' },
];

async function openExternalLink(url) {
  await openUrl(url);
}

async function handleOpenBootPatchFolder() {
  try {
    await invoke('open_boot_patch_folder');
  } catch (error) {
    console.error('Failed to open boot patch folder:', error);
    ElMessage.error(`打开失败: ${error}`);
  }
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

const props = defineProps({
  modeState: {
    type: Object,
    default: () => ({}),
  },
  executionState: {
    type: Object,
    default: () => ({}),
  },
  patchFlags: {
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
const patchFlags = computed(() => props.patchFlags || {});
const sourceState = computed(() => props.sourceState || {});
const toolState = computed(() => props.toolState || {});
const kernelSuState = computed(() => props.kernelSuState || {});
const rootState = computed(() => props.rootState || {});
const patchMode = computed(() => modeState.value.patchMode || 'magisk');
const patchModeOptions = computed(() => modeState.value.patchModeOptions || []);
const canPatch = computed(() => executionState.value.canPatch === true);
const canRoot = computed(() => executionState.value.canRoot === true);
const kernelSuRuntimeLoading = computed(() => executionState.value.kernelSuRuntimeLoading === true);
const patching = computed(() => executionState.value.patching === true);
const rooting = computed(() => executionState.value.rooting === true);
const isConnected = computed(() => executionState.value.isConnected === true);
const keepVerity = computed(() => patchFlags.value.keepVerity === true);
const keepForceEncrypt = computed(() => patchFlags.value.keepForceEncrypt === true);
const patchVbmetaFlag = computed(() => patchFlags.value.patchVbmetaFlag === true);
const recoveryMode = computed(() => patchFlags.value.recoveryMode === true);
const kernelSuAllowShell = computed(() => patchFlags.value.kernelSuAllowShell === true);
const kernelSuEnableAdbd = computed(() => patchFlags.value.kernelSuEnableAdbd === true);
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
const rootTargetPartition = computed(() => rootState.value.rootTargetPartition || '');
const rootPatchToolLabel = computed(() => rootState.value.rootPatchToolLabel || '');

const emit = defineEmits([
  'update:keep-verity',
  'update:keep-force-encrypt',
  'update:patch-vbmeta-flag',
  'update:recovery-mode',
  'update:patch-mode',
  'update:kernel-su-allow-shell',
  'update:kernel-su-enable-adbd',
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

const currentMode = computed(() => normalizePatchMode(patchMode.value));
const isKernelSuMode = computed(() => ['kernelsu', 'kernelsu_next', 'sukisu_ultra', 'resukisu'].includes(currentMode.value));
const isApatchMode = computed(() => ['apatch', 'folkpatch'].includes(currentMode.value));
const currentPatchModeLabel = computed(() => (
  patchModeOptions.value.find((item) => item.value === patchMode.value)?.label
  || (isKernelSuMode.value ? 'KernelSU' : (isApatchMode.value ? 'APatch' : 'Magisk'))
));
const readinessChips = computed(() => ([
  { key: 'device', label: 'ADB 设备', icon: 'device', ready: isConnected.value },
  { key: 'boot', label: 'Boot 来源', icon: 'target', ready: Boolean(bootPath.value) },
  {
    key: 'tool',
    label: currentPatchModeLabel.value,
    icon: 'package',
    ready: Boolean(
      isKernelSuMode.value
        ? kernelSuPath.value
        : (isApatchMode.value ? apatchApkPath.value : magiskApkPath.value)
    ),
  },
  { key: 'output', label: '输出目录', icon: 'folder', ready: Boolean(outputDir.value) },
]));

const riskNotice = computed(() => {
  if (patching.value) {
    if (isKernelSuMode.value) return 'KernelSU 修补期间请勿断开 USB、重启手机或切换设备模式。APK 资源、镜像推送与手机端 ksud 执行任一步骤被中断，都可能导致产物不完整。';
    if (isApatchMode.value) return 'APatch 修补期间请勿断开 USB、重启手机或切换设备模式。SuperKey 填错或资源不匹配都可能导致产物不可用。';
    return `${currentPatchModeLabel.value} 修补期间请勿断开 USB、重启手机或切换设备模式。任何中断都可能导致修补结果异常，后续刷入风险需自行判断。`;
  }

  if (rooting.value) {
    return '一键 Root 期间请保持 USB 连接稳定，不要手动切换 ADB / Fastboot / FastbootD 模式。若设备已自动进入刷入流程，请等待程序完成后再操作。';
  }

  if (isKernelSuMode.value) {
    return 'KernelSU 修补与后续刷入仍属于高风险操作，可能导致无法开机、功能异常或数据丢失。请务必提前备份原始镜像与重要数据。';
  }

  if (isApatchMode.value) {
    return 'APatch 会直接修改 boot 镜像中的内核补丁内容，风险不低于其它 Root 方案。请务必提前备份原始 boot 镜像、确认设备支持并妥善保管 SuperKey。';
  }

  return 'Boot 修补、Root 与后续刷入均属于高风险操作，可能引发卡开机、功能异常、数据丢失甚至变砖。请务必提前备份重要数据与原始镜像。';
});

const switchSectionDescription = computed(() => {
  if (isKernelSuMode.value) {
    return 'KernelSU 模式会自动从对应 APK 中提取手机端 `ksud` 与 `magiskboot`，并保留与官方 `ksud boot-patch` 对应的常用开关。';
  }

  if (isApatchMode.value) {
    return 'APatch 模式当前不额外暴露修补开关，核心参数为 APK 版本与 SuperKey。';
  }

  return '根据设备现状决定是否保留 verity、加密策略以及 Recovery 模式处理。';
});

const patchOptionChecklist = computed(() => (
  isKernelSuMode.value
    ? [
      {
        key: 'kernel-su-allow-shell',
        label: 'ALLOW SHELL',
        description: '允许 shell 默认拿 root',
        checked: kernelSuAllowShell.value,
        emitName: 'update:kernel-su-allow-shell',
      },
      {
        key: 'kernel-su-enable-adbd',
        label: 'ENABLE ADBD',
        description: '强制启用 adbd，并关闭 adbd auth',
        checked: kernelSuEnableAdbd.value,
        emitName: 'update:kernel-su-enable-adbd',
      },
    ]
    : isApatchMode.value
      ? []
      : [
      {
        key: 'keep-verity',
        label: 'KEEPVERITY',
        description: '保留 verity',
        checked: keepVerity.value,
        emitName: 'update:keep-verity',
      },
      {
        key: 'keep-force-encrypt',
        label: 'KEEPFORCEENCRYPT',
        description: '保留强制加密',
        checked: keepForceEncrypt.value,
        emitName: 'update:keep-force-encrypt',
      },
      {
        key: 'patch-vbmeta-flag',
        label: 'PATCHVBMETAFLAG',
        description: '处理 vbmeta flag',
        checked: patchVbmetaFlag.value,
        emitName: 'update:patch-vbmeta-flag',
      },
      {
        key: 'recovery-mode',
        label: 'RECOVERYMODE',
        description: 'Recovery 模式',
        checked: recoveryMode.value,
        emitName: 'update:recovery-mode',
      },
      ]
));

function handlePatchModeCommand(value) {
  emit('update:patch-mode', value);
}

function toggleOption(item) {
  if (patching.value || rooting.value) return;
  emit(item.emitName, !item.checked);
}
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.boot-hero-panel {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.hero-subbar__tips {
  min-width: 0;
}

.hero-aside-label,
.check-card__content span {
  display: block;
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.hero-subbar {
  align-items: flex-start;
  padding: 10px 14px;
  gap: 12px;
  --page-enter-delay: 0ms;
}

.hero-chip {
  gap: 8px;
  padding-inline: 12px;
  min-height: 32px;
}

.hero-tip-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  padding: 0 12px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(var(--color-info-rgb), 0.18);
  background: rgba(var(--color-info-rgb), 0.08);
  color: var(--color-info);
  font-size: 12px;
  font-weight: 500;
}

.hero-tip-chip--action {
  cursor: pointer;
  appearance: none;
}

.hero-tip-chip--action:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--color-primary-rgb), 0.22);
  background: rgba(var(--color-primary-rgb), 0.1);
  color: var(--color-primary);
}

.dropdown-caret {
  opacity: 0.72;
}

.hero-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.72fr) minmax(320px, 0.78fr);
  gap: 16px;
  align-items: stretch;
}

.hero-main {
  display: flex;
  flex-direction: column;
  --page-enter-delay: 40ms;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}

.hero-console {
  display: flex;
  flex-direction: column;
}

.hero-form-shell {
  min-width: 0;
}

.hero-aside {
  display: flex;
  --page-enter-delay: 80ms;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}

.hero-aside-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  padding: 18px;
}

.hero-risk-card__label,
.hero-switch-section__title {
  display: block;
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

.hero-risk-card {
  padding: 10px 12px;
  border-radius: 16px;
  border: 1px solid rgba(var(--color-warning-rgb), 0.18);
  background: color-mix(in srgb, var(--surface-panel) 94%, var(--surface-panel-strong) 6%);
}

.hero-risk-card p {
  margin-top: 6px;
  color: color-mix(in srgb, var(--color-text-secondary) 88%, var(--color-text-primary) 12%);
  font-size: 13px;
  line-height: 1.6;
}

.hero-switch-section {
  display: flex;
  flex-direction: column;
  gap: 26px;
  margin-top: 0;
  padding-top: 24px;
}

.hero-switch-section__desc {
  margin: -2px 0 0;
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.55;
}

.hero-checklist {
  display: grid;
  grid-template-columns: 1fr;
  gap: 6px;
  align-items: stretch;
}

.check-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
  min-height: 56px;
  padding: 8px 10px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--border-soft) 82%, var(--border-strong) 18%);
  background: color-mix(in srgb, var(--surface-panel) 92%, var(--surface-panel-strong) 8%);
  cursor: pointer;
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 300ms);
  transition: transform 0.24s ease, border-color 0.24s ease, box-shadow 0.24s ease, opacity 0.24s ease, background 0.24s ease;
}

.hero-checklist > :nth-child(1) {
  --page-enter-delay: 0ms;
}

.hero-checklist > :nth-child(2) {
  --page-enter-delay: 40ms;
}

.hero-checklist > :nth-child(3) {
  --page-enter-delay: 80ms;
}

.hero-checklist > :nth-child(4) {
  --page-enter-delay: 120ms;
}

.check-card:hover {
  transform: translateY(-2px);
  border-color: rgba(var(--color-primary-rgb), 0.22);
  box-shadow: var(--shadow-sm);
}

.check-card:focus-visible {
  outline: 2px solid rgba(var(--color-primary-rgb), 0.32);
  outline-offset: 2px;
}

.check-card.is-checked {
  border-color: rgba(var(--color-primary-rgb), 0.28);
  background: color-mix(in srgb, var(--surface-panel) 90%, var(--surface-panel-strong) 10%);
}

.check-card.is-disabled {
  cursor: not-allowed;
  opacity: 0.74;
}

.check-card.is-disabled:hover {
  transform: none;
  border-color: color-mix(in srgb, var(--border-soft) 82%, var(--border-strong) 18%);
  box-shadow: none;
}

.check-card__mark {
  position: relative;
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  border-radius: 6px;
  border: 1px solid rgba(var(--color-primary-rgb), 0.18);
  background: var(--surface-panel);
}

.check-card__mark.is-checked {
  border-color: transparent;
  background: var(--color-primary);
}

.check-card__tick {
  position: absolute;
  top: 2px;
  left: 5px;
  width: 4px;
  height: 8px;
  border-right: 2px solid transparent;
  border-bottom: 2px solid transparent;
  transform: rotate(45deg);
  transition: border-color 0.2s ease;
}

.check-card__mark.is-checked .check-card__tick {
  border-color: var(--text-on-primary);
}

.check-card__content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.check-card__content span {
  color: var(--color-text-primary);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.06em;
  line-height: 1.2;
}

.check-card__content strong {
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.3;
  word-break: break-word;
}

.boot-usage-dialog {
  .el-dialog__header {
    padding-bottom: 6px;
  }

  .el-dialog__title {
    font-size: 18px;
    font-weight: 700;
  }

  .el-dialog__body {
    padding-top: 10px;
  }
}

.usage-dialog {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.usage-note {
  padding: 16px 18px;
  border-radius: 20px;
  border: 1px solid rgba(var(--color-primary-rgb), 0.14);
  background: linear-gradient(135deg, rgba(var(--color-primary-rgb), 0.08), rgba(var(--color-info-rgb), 0.08));
}

.usage-note__eyebrow {
  display: block;
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.usage-note h3 {
  margin: 8px 0 10px;
  color: var(--color-text-primary);
  font-size: 20px;
  line-height: 1.35;
}

.usage-note p {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 28px;
}

.usage-link {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-primary);
  font: inherit;
  text-decoration: none;
  cursor: pointer;
}

.usage-link:hover {
  color: var(--color-primary-hover);
  text-decoration: underline;
}

.manager-link-list {
  margin-top: 6px;
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 28px;
}

.manager-link {
  line-height: inherit;
}

.manager-link-separator {
  color: var(--color-text-secondary);
}

@media (max-width: 1180px) {
  .hero-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  .hero-subbar {
    flex-direction: column;
    align-items: stretch;
  }

  .hero-tip-chip {
    width: 100%;
    justify-content: center;
  }
}

@media (max-width: 520px) {
  .hero-subbar__tips {
    width: 100%;
  }
}
</style>




