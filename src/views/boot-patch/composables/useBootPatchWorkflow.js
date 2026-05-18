import { computed, h, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  bootPatchOneKeyRoot,
  generateApatchSuperKey as requestApatchSuperKey,
  patchBootImage,
  prepareBootPatchAutoSource,
} from '@/api/bootPatch';
import {
  APATCH_SUPER_KEY_MAX_LENGTH,
  APATCH_SUPER_KEY_MIN_LENGTH,
  getFileTail,
  getPatchModeLabel,
  isApatchMode,
  isHttpUrl,
  isKernelSuMode,
  normalizePatchMode,
} from './bootPatchMode';

export function useBootPatchWorkflow({
  form,
  isConnected,
  patching,
  rooting,
  defaultOutputDir,
  floatingLogRef,
  kernelSuKmiOptions,
  kernelSuSuggestedKmi,
  getApatchToolOptionsByMode,
  getMagiskToolOptionsByMode,
  getKernelToolOptionsByMode,
  getDefaultMagiskToolPath,
  getDefaultApatchToolPath,
  getDefaultKernelSuToolPath,
  getEffectivePatchToolPath,
  getInitialPatchMode,
}) {
  const logs = ref([]);
  const status = ref('idle');
  const lastProgressStatus = ref('idle');
  const outputPath = ref('');
  const packageZipPath = ref('');
  const imageMetaPreview = ref('');
  const remoteWorkDir = ref('');
  const latestPatchResult = ref(null);
  const rootAppInstallWarningShown = ref(false);
  let apatchSuperKeyRequestId = 0;

  const selectedPatchToolPath = computed(() => (
    getEffectivePatchToolPath(form.patchMode)
  ));
  const selectedKernelSuApkPath = computed(() => (
    getKernelSuApkPathByBinaryPath(form.kernelSuPath)
  ));
  const canPatch = computed(() => {
    if (!selectedPatchToolPath.value || !form.outputDir) {
      return false;
    }

    if (isApatchMode(form.patchMode) && !String(form.apatchSuperKey || '').trim()) {
      return false;
    }

    return isConnected.value;
  });
  const canRoot = computed(() => {
    if (!selectedPatchToolPath.value || !form.outputDir || !isConnected.value) {
      return false;
    }

    if (isKernelSuMode(form.patchMode)) {
      return Boolean(selectedKernelSuApkPath.value);
    }

    if (isApatchMode(form.patchMode)) {
      return Boolean(String(form.apatchSuperKey || '').trim());
    }

    return true;
  });
  const rootTargetPartition = computed(() => {
    const partition = latestPatchResult.value?.targetPartition || '';
    const slotSuffix = latestPatchResult.value?.targetSlotSuffix || '';
    return partition ? `${partition}${slotSuffix}` : '';
  });
  const rootPatchToolLabel = computed(() => latestPatchResult.value?.toolLabel || '');
  const activePatchMode = computed(() => normalizePatchMode(latestPatchResult.value?.patchMode || form.patchMode));
  const patchSteps = computed(() => {
    const definitions = isKernelSuMode(activePatchMode.value)
      ? [
        { key: 'prepare', label: '准备参数', desc: '校验输入、解析镜像来源并确定输出文件名' },
        { key: 'patch', label: '执行修补', desc: `调用手机端 ksud boot-patch 完成 ${getPatchModeLabel(activePatchMode.value)} 修补` },
        { key: 'pack', label: '整理资料包', desc: '生成包含管理器 APK、原版镜像、platform-tools、脚本与说明的资料包' },
        { key: 'clean', label: '清理环境', desc: '清理 Payload 提取目录等本地临时文件' },
      ]
      : isApatchMode(activePatchMode.value)
        ? [
          { key: 'prepare', label: '准备参数', desc: '校验输入参数并初始化工作目录' },
          { key: 'check', label: '校验环境', desc: `检查设备连接、ABI 与 ${getPatchModeLabel(activePatchMode.value)} 修补资源完整性` },
          { key: 'push', label: '推送文件', desc: `将 ${getPatchModeLabel(activePatchMode.value)} 修补脚本、二进制与镜像推送到手机临时目录` },
          { key: 'patch', label: '执行修补', desc: `调用 ${getPatchModeLabel(activePatchMode.value)} boot_patch.sh 与 kptools 对 boot 镜像进行处理` },
          { key: 'pull', label: '回传结果', desc: '将 patched 镜像拉回到本地输出目录' },
          { key: 'pack', label: '整理资料包', desc: '生成包含管理器 APK、原版镜像、platform-tools、脚本与说明的资料包' },
          { key: 'clean', label: '清理环境', desc: '移除手机与本地临时目录中的中间文件' },
        ]
        : [
          { key: 'prepare', label: '准备参数', desc: '校验输入参数并初始化工作目录' },
          { key: 'check', label: '校验环境', desc: '检查设备连接、依赖与镜像来源' },
          { key: 'push', label: '推送文件', desc: '将修补脚本与镜像推送到手机临时目录' },
          { key: 'patch', label: '执行修补', desc: '调用 Magisk boot_patch.sh 对 boot 镜像进行处理' },
          { key: 'pull', label: '回传结果', desc: '将 patched 镜像拉回到本地输出目录' },
          { key: 'pack', label: '整理资料包', desc: '生成包含管理器 APK、原版镜像、platform-tools、脚本与说明的资料包' },
          { key: 'clean', label: '清理环境', desc: '移除手机与本地临时目录中的中间文件' },
        ];

    if (isKernelSuMode(activePatchMode.value)) {
      definitions.splice(0, definitions.length, ...[
        { key: 'prepare', label: '准备参数', desc: '校验输入参数并初始化工作目录' },
        { key: 'check', label: '校验环境', desc: `检查设备连接、ABI 与 ${getPatchModeLabel(activePatchMode.value)} 修补资源完整性` },
        { key: 'push', label: '推送文件', desc: `将 ${getPatchModeLabel(activePatchMode.value)} APK 内的 ksud、magiskboot 与镜像推送到手机临时目录` },
        { key: 'patch', label: '执行修补', desc: '调用手机端 ksud boot-patch 处理 boot / init_boot 镜像' },
        { key: 'pull', label: '回传结果', desc: `将 ${getPatchModeLabel(activePatchMode.value)} patched 镜像拉回到本地输出目录` },
        { key: 'pack', label: '整理资料包', desc: '生成包含管理器 APK、原版镜像、platform-tools、脚本与说明的资料包' },
        { key: 'clean', label: '清理环境', desc: '移除手机与本地临时目录中的中间文件' },
      ]);
    }

    if (status.value === 'idle') {
      return definitions.map((step) => ({ ...step, state: 'idle' }));
    }

    if (status.value === 'done') {
      return definitions.map((step) => ({ ...step, state: 'done' }));
    }

    const activeStatus = status.value === 'error' ? lastProgressStatus.value : status.value;
    const activeIndex = definitions.findIndex((step) => step.key === activeStatus);

    return definitions.map((step, index) => {
      if (activeIndex === -1) {
        return { ...step, state: 'idle' };
      }

      if (status.value === 'error') {
        if (index < activeIndex) return { ...step, state: 'done' };
        if (index === activeIndex) return { ...step, state: 'error' };
        return { ...step, state: 'idle' };
      }

      if (index < activeIndex) return { ...step, state: 'done' };
      if (index === activeIndex) return { ...step, state: 'active' };
      return { ...step, state: 'idle' };
    });
  });

  function addLocalLog(content, type = 'info', tag = 'UI') {
    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });
    logs.value.push({ time, content, type, tag });
  }

  async function openPackageZipDir(packagePath) {
    try {
      const normalizedPath = String(packagePath || '').trim();
      if (!normalizedPath) {
        ElMessage.warning('当前没有可打开的资料包目录');
        return;
      }

      const normalizedDir = normalizedPath.replace(/[\\/]+$/, '').replace(/[\\/][^\\/]+$/, '');
      await invoke('open_folder_path', { path: normalizedDir });
    } catch (error) {
      ElMessage.error(`打开所在目录失败：${error}`);
    }
  }

  function setStatus(nextStatus) {
    status.value = nextStatus;
    if (nextStatus !== 'idle' && nextStatus !== 'error') {
      lastProgressStatus.value = nextStatus;
    }
  }

  function getMagiskLabelByPath(apkPath) {
    const matched = getMagiskToolOptionsByMode(form.patchMode).find((item) => item.value === apkPath);
    return matched?.label || getFileTail(apkPath) || '未选择';
  }

  function getApatchLabelByPath(apkPath) {
    const matched = getApatchToolOptionsByMode(form.patchMode).find((item) => item.value === apkPath);
    return matched?.label || getFileTail(apkPath) || '未选择';
  }

  function getKernelSuLabelByPath(binaryPath) {
    const matched = getKernelToolOptionsByMode(form.patchMode).find((item) => item.value === binaryPath);
    return matched?.label || getFileTail(binaryPath) || '未选择';
  }

  function getPatchToolLabel(mode, toolPath) {
    if (isKernelSuMode(mode)) return getKernelSuLabelByPath(toolPath);
    if (isApatchMode(mode)) return getApatchLabelByPath(toolPath);
    return getMagiskLabelByPath(toolPath);
  }

  function getRootManagerLabel(mode) {
    return getPatchModeLabel(mode);
  }

  function getKernelSuApkPathByBinaryPath(binaryPath) {
    const matched = getKernelToolOptionsByMode(form.patchMode).find((item) => item.value === binaryPath);
    return matched?.apkPath || '';
  }

  function getKernelSuKmiValidationMessage(mode = form.patchMode) {
    if (!isKernelSuMode(mode)) {
      return '';
    }

    const selectedKmi = String(form.kernelSuKmi || '').trim();
    const supportedKmis = kernelSuKmiOptions.value
      .map((item) => String(item?.value || '').trim())
      .filter(Boolean);

    if (!supportedKmis.length) {
      return '当前 KernelSU 版本暂未读取到可用的 KMI 列表，请稍后重试或重新选择版本后再继续。';
    }

    if (!selectedKmi) {
      return '请选择下拉列表中的有效 KMI 后再继续。';
    }

    if (!supportedKmis.includes(selectedKmi)) {
      return `当前 KMI“${selectedKmi}”不在所选 KernelSU 版本支持列表中，请从下拉选项中重新选择后再继续。`;
    }

    return '';
  }

  function ensureKernelSuKmiSelection(mode = form.patchMode) {
    const message = getKernelSuKmiValidationMessage(mode);
    if (!message) {
      return true;
    }

    addLocalLog(message, 'warning', 'UI');
    ElMessage.warning(message);
    return false;
  }

  function getOnlineRomSourceValidationMessage(value) {
    const normalized = String(value || '').trim();
    if (!isHttpUrl(normalized)) {
      return '';
    }

    const lowerCaseValue = normalized.toLowerCase();
    const pureUrl = lowerCaseValue.split(/[?#]/)[0] || lowerCaseValue;
    if (pureUrl.endsWith('.tgz') || lowerCaseValue.includes('fw_')) {
      return '当前在线地址疑似线刷包（.tgz / fw_），Boot 修补不支持直接选择线刷包，请改用卡刷包、payload 或 boot 镜像';
    }

    return '';
  }

  function isPayloadLikeBootSource(value) {
    const normalized = String(value || '').trim();
    if (!normalized) {
      return false;
    }

    if (isHttpUrl(normalized)) {
      return true;
    }

    const extension = normalized.split('.').pop()?.toLowerCase() || '';
    return extension === 'bin' || extension === 'zip';
  }

  function getApatchBootSourceValidationMessage(value, mode = form.patchMode) {
    const normalized = String(value || '').trim();
    if (!normalized || isPayloadLikeBootSource(normalized)) {
      return '';
    }

    if (getFileTail(normalized).toLowerCase().includes('init_boot')) {
      return `${getPatchModeLabel(mode)} 官方仅支持 boot.img，本地 init_boot.img 不能用于修补`;
    }

    return '';
  }

  function getApatchSuperKeyValidationMessage(value, mode = form.patchMode) {
    const normalized = String(value || '').trim();
    const patchLabel = getPatchModeLabel(mode);

    if (!normalized) {
      return `${patchLabel} SuperKey 不能为空`;
    }

    if (normalized.length < APATCH_SUPER_KEY_MIN_LENGTH || normalized.length > APATCH_SUPER_KEY_MAX_LENGTH) {
      return `${patchLabel} SuperKey 需为 ${APATCH_SUPER_KEY_MIN_LENGTH}-${APATCH_SUPER_KEY_MAX_LENGTH} 位`;
    }

    if (!/^[A-Za-z0-9]+$/.test(normalized)) {
      return `${patchLabel} SuperKey 只能包含字母和数字`;
    }

    return '';
  }

  function ensureApatchSuperKeySelection(mode = form.patchMode) {
    if (!isApatchMode(mode)) {
      return true;
    }

    const message = getApatchSuperKeyValidationMessage(form.apatchSuperKey, mode);
    if (!message) {
      return true;
    }

    addLocalLog(message, 'warning', 'UI');
    ElMessage.warning(message);
    return false;
  }

  function ensureOnlineRomSourceSelection(value = form.bootPath) {
    const message = getOnlineRomSourceValidationMessage(value);
    if (!message) {
      return true;
    }

    addLocalLog(message, 'warning', 'UI');
    ElMessage.warning(message);
    return false;
  }

  function ensureApatchBootSourceSelection(mode = form.patchMode) {
    if (!isApatchMode(mode)) {
      return true;
    }

    const message = getApatchBootSourceValidationMessage(form.bootPath, mode);
    if (!message) {
      return true;
    }

    addLocalLog(message, 'warning', 'UI');
    ElMessage.warning(message);
    return false;
  }

  async function ensureApatchSuperKeyFilled(mode = form.patchMode, options = {}) {
    if (!isApatchMode(mode)) {
      return;
    }

    if (String(form.apatchSuperKey || '').trim()) {
      return;
    }

    const requestId = ++apatchSuperKeyRequestId;

    try {
      const generatedSuperKey = await requestApatchSuperKey();
      if (requestId !== apatchSuperKeyRequestId) {
        return;
      }

      if (!isApatchMode(form.patchMode) || String(form.apatchSuperKey || '').trim()) {
        return;
      }

      form.apatchSuperKey = String(generatedSuperKey || '').trim();
      if (!options.silent) {
        addLocalLog('已由后端按 APatch 官方要求自动生成一份 SuperKey，可继续手动修改', 'success', 'UI');
      }
    } catch (error) {
      if (requestId !== apatchSuperKeyRequestId) {
        return;
      }

      addLocalLog(`后端生成 APatch SuperKey 失败：${error}`, 'warning', 'UI');
      if (!options.silent) {
        ElMessage.warning(`后端生成 APatch SuperKey 失败：${error}`);
      }
    }
  }

  function syncStatusByTag(tag, type) {
    if (status.value === 'done' && tag === 'CLEAN') {
      return;
    }

    if (type === 'error') {
      setStatus('error');
      return;
    }

    if (tag === 'PREP') setStatus('prepare');
    if (tag === 'CHK') setStatus('check');
    if (tag === 'PUSH') setStatus('push');
    if (tag === 'PATCH') setStatus('patch');
    if (tag === 'PULL') setStatus('pull');
    if (tag === 'PACK') setStatus('pack');
    if (tag === 'CLEAN') setStatus('clean');
    if (tag === 'DONE') setStatus('done');
  }

  async function selectBootFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Boot / Payload', extensions: ['img', 'bin', 'zip', 'lz4'] }],
    });
    if (!selected) return;

    const apatchBootSourceError = isApatchMode(form.patchMode)
      ? getApatchBootSourceValidationMessage(selected)
      : '';
    if (apatchBootSourceError) {
      addLocalLog(apatchBootSourceError, 'warning', 'UI');
      ElMessage.warning(apatchBootSourceError);
      return;
    }

    form.bootPath = selected;

    addLocalLog(`已选择 Boot 文件：${selected}`);
  }

  async function selectOutputDir() {
    const selected = await open({ multiple: false, directory: true });
    if (!selected) return;

    form.outputDir = selected;
    addLocalLog(`已选择输出目录：${selected}`);
  }

  function prepareRootLogSession(message) {
    latestPatchResult.value = null;
    outputPath.value = '';
    packageZipPath.value = '';
    imageMetaPreview.value = '';
    remoteWorkDir.value = '';
    logs.value = [];
    status.value = 'idle';
    lastProgressStatus.value = 'idle';
    floatingLogRef.value?.open?.();
    addLocalLog(message, 'info', 'UI');
  }

  async function ensureRootBootSourceReady() {
    if (String(form.bootPath || '').trim()) {
      return false;
    }

    prepareRootLogSession('未填写 Boot 文件或官方链接，开始尝试从 ROM 下载页的四个站点自动匹配对应版本。');

    const result = await prepareBootPatchAutoSource();
    const nextBootPath = String(result?.bootPath || '').trim();
    if (!nextBootPath) {
      throw new Error('未从四个 ROM 站点匹配到可用于修补的官方链接');
    }

    form.bootPath = nextBootPath;

    const sourceLabel = String(result?.sourceLabel || result?.sourceKey || '未知来源').trim();
    addLocalLog(`已自动填入官方链接（${sourceLabel}）：${nextBootPath}`, 'success', 'UI');
    return true;
  }

  async function startPatch() {
    if (patching.value || rooting.value) return;

    if (!canPatch.value) {
      const patchLabel = getPatchModeLabel(form.patchMode);
      ElMessage.warning(
        isKernelSuMode(form.patchMode)
          ? `请先补齐 Boot 文件、${patchLabel} APK 和输出目录，并保持设备已连接`
          : isApatchMode(form.patchMode)
            ? `请先补齐 Boot 文件、${patchLabel} APK、SuperKey 和输出目录，并保持设备已连接`
            : '请先补齐 Boot 文件、Magisk APK 和输出目录，并保持设备已连接',
      );
      return;
    }

    if (!ensureKernelSuKmiSelection(form.patchMode)) {
      return;
    }

    if (!ensureApatchSuperKeySelection(form.patchMode)) {
      return;
    }

    patching.value = true;

    try {
      const autoFilledBootSource = await ensureRootBootSourceReady();

      if (!ensureOnlineRomSourceSelection()) {
        return;
      }

      if (!ensureApatchBootSourceSelection(form.patchMode)) {
        return;
      }

      const result = await runPatchPipeline({
        launchMode: 'patch',
        preserveLogs: autoFilledBootSource,
      });
      addLocalLog(
        result.packageZipFileName
          ? `修补完成，已生成镜像 ${result.outputFileName} 与资料包 ${result.packageZipFileName}`
          : `修补完成，输出文件：${result.outputFileName}`,
        'success',
        'UI',
      );
    } catch (error) {
      ElMessage.error(`Boot 修补失败：${error}`);
    } finally {
      patching.value = false;
    }
  }

  async function startOneKeyRoot() {
    if (patching.value || rooting.value) return;

    if (!isConnected.value) {
      ElMessage.warning('当前没有可用的 ADB 设备连接');
      return;
    }

    if (!canRoot.value) {
      const patchLabel = getPatchModeLabel(form.patchMode);
      ElMessage.warning(
        isKernelSuMode(form.patchMode)
          ? `请先补齐 Boot 文件、${patchLabel} APK 和输出目录，并保持设备已连接`
          : isApatchMode(form.patchMode)
            ? `请先补齐 Boot 文件、${patchLabel} APK、SuperKey 和输出目录，并保持设备已连接`
            : '请先补齐 Boot 文件、Magisk APK 和输出目录，并保持设备已连接',
      );
      return;
    }

    if (!ensureKernelSuKmiSelection(form.patchMode)) {
      return;
    }

    if (!ensureApatchSuperKeySelection(form.patchMode)) {
      return;
    }

    try {
      const patchLabel = getPatchModeLabel(form.patchMode);
      const confirmMessage = isKernelSuMode(form.patchMode)
        ? `将先执行 ${patchLabel} 修补，再安装 ${getFileTail(selectedKernelSuApkPath.value) || `对应 ${patchLabel} 安装包`}，随后自动重启到 Fastboot 刷入修补结果；如果设备更适合 FastbootD，会自动切换重试。`
        : isApatchMode(form.patchMode)
          ? `将先执行 ${patchLabel} 修补，再安装 ${getApatchLabelByPath(form.apatchApkPath)}，随后自动重启到 Fastboot 刷入修补结果；如果设备更适合 FastbootD，会自动切换重试。`
          : `将先执行修补，再安装 ${getMagiskLabelByPath(form.magiskApkPath)}，随后自动重启到 Fastboot 刷入修补结果；如果设备更适合 FastbootD，会自动切换重试。`;

      await ElMessageBox.confirm(confirmMessage, '一键 Root 确认', {
        confirmButtonText: '继续执行',
        cancelButtonText: '取消',
        type: 'warning',
      });
    } catch {
      return;
    }

    rooting.value = true;
    rootAppInstallWarningShown.value = false;

    try {
      const autoFilledBootSource = await ensureRootBootSourceReady();

      if (!ensureOnlineRomSourceSelection()) {
        return;
      }

      if (!ensureApatchBootSourceSelection(form.patchMode)) {
        return;
      }

      const patchResult = await runPatchPipeline({
        launchMode: 'root',
        preserveLogs: autoFilledBootSource,
      });

      addLocalLog(
        isKernelSuMode(form.patchMode)
          ? `修补完成，开始安装 ${getPatchModeLabel(form.patchMode)} APK 并执行自动刷入`
          : isApatchMode(form.patchMode)
            ? `修补完成，开始安装 ${getPatchModeLabel(form.patchMode)} APK 并执行自动刷入`
            : '修补完成，开始安装 Magisk 并执行自动刷入',
        'info',
        'UI',
      );

      const result = await bootPatchOneKeyRoot({
        patchMode: form.patchMode,
        patchedImagePath: patchResult.outputPath,
        magiskApkPath: isKernelSuMode(form.patchMode) || isApatchMode(form.patchMode) ? '' : form.magiskApkPath,
        apatchApkPath: isApatchMode(form.patchMode) ? form.apatchApkPath : '',
        kernelSuApkPath: isKernelSuMode(form.patchMode) ? selectedKernelSuApkPath.value : '',
        targetPartition: patchResult.targetPartition,
        targetSlotSuffix: patchResult.targetSlotSuffix || '',
      });

      const isKernelSuRoot = isKernelSuMode(form.patchMode);
      const isApatchRoot = isApatchMode(form.patchMode);
      const rootManagerLabel = getRootManagerLabel(form.patchMode);
      const installSucceeded = isKernelSuRoot
        ? result.kernelSuInstallSucceeded !== false
        : isApatchRoot
          ? result.apatchInstallSucceeded !== false
          : result.magiskInstallSucceeded !== false;
      const installError = isKernelSuRoot
        ? result.kernelSuInstallError
        : isApatchRoot
          ? result.apatchInstallError
          : result.magiskInstallError;
      const installedPath = isKernelSuRoot
        ? (result.installedKernelSuPath || selectedKernelSuApkPath.value)
        : isApatchRoot
          ? (result.installedApatchPath || form.apatchApkPath)
          : (result.installedMagiskPath || form.magiskApkPath);

      if (!installSucceeded) {
        addLocalLog(
          `${rootManagerLabel} APK 自动安装失败，已继续完成刷入流程：${installError || '未知原因'}`,
          'warning',
          'UI',
        );
        if (!rootAppInstallWarningShown.value) {
          await showRootAppInstallFallbackDialog({
            appLabel: rootManagerLabel,
            installedPath,
            installError,
            flashedMode: result.flashedMode,
            flashedPartition: result.flashedPartition,
          }, { completed: true });
        } else {
          ElMessage.success(`一键 Root 完成，已在 ${result.flashedMode} 模式刷入 ${result.flashedPartition} 并重启`);
        }
        return;
      }

      ElMessage.success(`一键 Root 完成，已在 ${result.flashedMode} 模式刷入 ${result.flashedPartition} 并重启`);
    } catch (error) {
      ElMessage.error(`一键 Root 失败：${error}`);
    } finally {
      rooting.value = false;
    }
  }

  async function showRootAppInstallFallbackDialog(payload, options = {}) {
    rootAppInstallWarningShown.value = true;
    const completed = options.completed === true;
    const appLabel = payload.appLabel || 'Root 管理器';
    const summaryText = completed
      ? `${appLabel} APK 自动安装失败，但后续 Root 刷入流程已经继续执行并完成，当前已在 ${payload.flashedMode} 模式刷入 ${payload.flashedPartition}。`
      : `${appLabel} APK 自动安装失败，但程序会继续执行后续刷入流程。你可以等待流程结束后，再自行安装下面这个 APK。`;

    await ElMessageBox.alert(
      h('div', { style: { display: 'flex', flexDirection: 'column', gap: '10px' } }, [
        h('p', { style: { margin: '0', lineHeight: '1.7' } }, summaryText),
        h('p', { style: { margin: '0', lineHeight: '1.7' } }, '你可以在设备开机后自行安装以下 APK：'),
        h(
          'div',
          {
            style: {
              padding: '10px 12px',
              borderRadius: '12px',
              background: 'rgba(var(--color-warning-rgb), 0.08)',
              border: '1px solid rgba(var(--color-warning-rgb), 0.18)',
              lineHeight: '1.6',
              wordBreak: 'break-all',
            },
          },
          payload.installedPath,
        ),
        ...(payload.installError
          ? [h('p', { style: { margin: '0', lineHeight: '1.7', color: 'var(--color-text-secondary)' } }, `失败原因：${payload.installError}`)]
          : []),
      ]),
      `${appLabel} APK 安装失败`,
      {
        confirmButtonText: '知道了',
        type: 'warning',
      },
    );
  }

  function buildPatchRequest() {
    const effectivePatchMode = normalizePatchMode(form.patchMode);
    return {
      patchMode: effectivePatchMode,
      bootPath: form.bootPath,
      magiskApkPath: !isKernelSuMode(effectivePatchMode) && !isApatchMode(effectivePatchMode)
        ? getEffectivePatchToolPath(effectivePatchMode)
        : form.magiskApkPath,
      apatchApkPath: isApatchMode(effectivePatchMode) ? getEffectivePatchToolPath(effectivePatchMode) : form.apatchApkPath,
      apatchSuperKey: isApatchMode(effectivePatchMode) ? form.apatchSuperKey : '',
      kernelSuPath: isKernelSuMode(effectivePatchMode) ? getEffectivePatchToolPath(effectivePatchMode) : form.kernelSuPath,
      kernelSuKmi: isKernelSuMode(effectivePatchMode) ? form.kernelSuKmi : '',
      outputDir: form.outputDir,
      keepVerity: form.keepVerity,
      keepForceEncrypt: form.keepForceEncrypt,
      patchVbmetaFlag: form.patchVbmetaFlag,
      recoveryMode: form.recoveryMode,
      kernelSuAllowShell: isKernelSuMode(effectivePatchMode) ? form.kernelSuAllowShell : false,
      kernelSuEnableAdbd: isKernelSuMode(effectivePatchMode) ? form.kernelSuEnableAdbd : false,
      cleanupRemote: form.cleanupRemote,
    };
  }

  function syncPatchResult(result) {
    const toolPath = getEffectivePatchToolPath(form.patchMode);
    outputPath.value = result.outputPath;
    packageZipPath.value = result.packageZipPath || '';
    remoteWorkDir.value = result.remoteWorkDir || '';
    latestPatchResult.value = {
      patchMode: normalizePatchMode(result.patchMode || form.patchMode),
      outputPath: result.outputPath,
      outputFileName: result.outputFileName,
      packageZipPath: result.packageZipPath || '',
      packageZipFileName: result.packageZipFileName || '',
      targetPartition: result.targetPartition,
      targetSlotSuffix: result.targetSlotSuffix || '',
      toolPath,
      toolLabel: getPatchToolLabel(form.patchMode, toolPath),
    };

    if (packageZipPath.value) {
      ElMessage.success('root线刷包已生成，可在执行日志中打开资料包（输出目录）并自行备份');
    }
  }

  async function runPatchPipeline({ launchMode, preserveLogs }) {
    const currentMode = normalizePatchMode(form.patchMode);

    if ((launchMode === 'root' || !isKernelSuMode(currentMode)) && !isConnected.value) {
      throw new Error('当前没有可用的 ADB 设备连接');
    }

    if ((launchMode === 'root' && !canRoot.value) || (launchMode !== 'root' && !canPatch.value)) {
      const patchLabel = getPatchModeLabel(currentMode);
      throw new Error(
        isKernelSuMode(currentMode)
          ? `请先补齐 Boot 文件、${patchLabel} APK 和输出目录`
          : isApatchMode(currentMode)
            ? `请先补齐 Boot 文件、${patchLabel} APK、SuperKey 和输出目录`
            : '请先补齐 Boot 文件、Magisk APK 和输出目录',
      );
    }

    if (!ensureApatchSuperKeySelection(currentMode)) {
      throw new Error(getApatchSuperKeyValidationMessage(form.apatchSuperKey) || 'APatch SuperKey 校验失败');
    }

    if (!ensureOnlineRomSourceSelection()) {
      throw new Error(getOnlineRomSourceValidationMessage(form.bootPath) || '在线 ROM 地址校验失败');
    }

    if (!ensureApatchBootSourceSelection(currentMode)) {
      throw new Error(getApatchBootSourceValidationMessage(form.bootPath) || 'APatch Boot 镜像校验失败');
    }

    latestPatchResult.value = null;
    setStatus('prepare');
    outputPath.value = '';
    packageZipPath.value = '';
    imageMetaPreview.value = '';
    remoteWorkDir.value = '';
    if (!preserveLogs) {
      logs.value = [];
    }
    floatingLogRef.value?.open?.();
    addLocalLog(
      launchMode === 'root'
        ? `开始执行一键 Root 流程，当前修补方案：${getPatchModeLabel(currentMode)}`
        : `开始执行 Boot 修补流程，当前修补方案：${getPatchModeLabel(currentMode)}`,
      'info',
      'UI',
    );

    try {
      const result = await patchBootImage(buildPatchRequest());
      syncPatchResult(result);
      setStatus('done');
      return result;
    } catch (error) {
      setStatus('error');
      addLocalLog(
        `${launchMode === 'root' ? '一键 Root 前置修补失败' : 'Boot 修补失败'}：${error}`,
        'error',
        'UI',
      );
      throw error;
    }
  }

  function resetForm() {
    form.patchMode = getInitialPatchMode();
    form.bootPath = '';
    form.magiskApkPath = getDefaultMagiskToolPath(form.patchMode);
    form.apatchApkPath = getDefaultApatchToolPath();
    form.apatchSuperKey = '';
    form.kernelSuPath = getDefaultKernelSuToolPath();
    form.kernelSuKmi = kernelSuSuggestedKmi.value || '';
    form.outputDir = defaultOutputDir.value;
    form.keepVerity = false;
    form.keepForceEncrypt = false;
    form.patchVbmetaFlag = false;
    form.recoveryMode = false;
    form.kernelSuAllowShell = false;
    form.kernelSuEnableAdbd = false;
    form.cleanupRemote = true;
    outputPath.value = '';
    packageZipPath.value = '';
    imageMetaPreview.value = '';
    remoteWorkDir.value = '';
    latestPatchResult.value = null;
    rootAppInstallWarningShown.value = false;
    status.value = 'idle';
    lastProgressStatus.value = 'idle';
    logs.value = [];
    void ensureApatchSuperKeyFilled(form.patchMode, { silent: true });
  }

  function handleBootPatchLog(payload) {
    const type = payload.logType || payload.log_type || 'info';
    const tag = payload.tag || 'PATCH';
    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false });

    logs.value.push({
      time,
      content: payload.content,
      type,
      tag,
    });

    syncStatusByTag(tag, type);

    if (tag === 'PUSH' && payload.content?.startsWith('创建手机临时目录: ')) {
      remoteWorkDir.value = payload.content.replace('创建手机临时目录: ', '');
    }

    if (tag === 'DONE' && payload.content?.startsWith('Boot 修补完成: ')) {
      outputPath.value = payload.content.replace('Boot 修补完成: ', '');
    }

    if (tag === 'PACK' && payload.content?.startsWith('刷机资料包已生成: ')) {
      packageZipPath.value = payload.content.replace('刷机资料包已生成: ', '');
    }

    if (tag === 'PACK' && payload.content?.startsWith('镜像解析预览: ')) {
      imageMetaPreview.value = payload.content.replace('镜像解析预览: ', '');
    }

    if (
      rooting.value
      && !rootAppInstallWarningShown.value
      && tag === 'ROOT'
      && typeof payload.content === 'string'
      && (
        payload.content.startsWith('安装 Magisk APK 失败')
        || payload.content.startsWith('安装 KernelSU APK 失败')
        || payload.content.startsWith('安装 APatch APK 失败')
        || payload.content.startsWith(`安装 ${getRootManagerLabel(form.patchMode)} APK 失败`)
      )
    ) {
      void showRootAppInstallFallbackDialog({
        appLabel: getRootManagerLabel(form.patchMode),
        installedPath: isKernelSuMode(form.patchMode)
          ? selectedKernelSuApkPath.value
          : (isApatchMode(form.patchMode) ? form.apatchApkPath : form.magiskApkPath),
        installError: payload.content,
      }, { completed: false });
    }
  }

  return {
    logs,
    status,
    lastProgressStatus,
    outputPath,
    packageZipPath,
    imageMetaPreview,
    remoteWorkDir,
    latestPatchResult,
    rootAppInstallWarningShown,
    selectedPatchToolPath,
    selectedKernelSuApkPath,
    canPatch,
    canRoot,
    rootTargetPartition,
    rootPatchToolLabel,
    activePatchMode,
    patchSteps,
    addLocalLog,
    openPackageZipDir,
    setStatus,
    getMagiskLabelByPath,
    getApatchLabelByPath,
    getKernelSuLabelByPath,
    getPatchToolLabel,
    getRootManagerLabel,
    getKernelSuApkPathByBinaryPath,
    getKernelSuKmiValidationMessage,
    ensureKernelSuKmiSelection,
    getOnlineRomSourceValidationMessage,
    getApatchBootSourceValidationMessage,
    getApatchSuperKeyValidationMessage,
    ensureApatchSuperKeySelection,
    ensureOnlineRomSourceSelection,
    ensureApatchBootSourceSelection,
    ensureApatchSuperKeyFilled,
    syncStatusByTag,
    selectBootFile,
    selectOutputDir,
    prepareRootLogSession,
    ensureRootBootSourceReady,
    startPatch,
    startOneKeyRoot,
    showRootAppInstallFallbackDialog,
    buildPatchRequest,
    syncPatchResult,
    runPatchPipeline,
    resetForm,
    handleBootPatchLog,
  };
}
