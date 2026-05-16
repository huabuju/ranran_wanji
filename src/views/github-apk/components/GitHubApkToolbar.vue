<template>
  <div class="top-action-bar page-toolbar surface-card github-apk-toolbar">
    <div class="toolbar-row toolbar-row--primary">
      <div class="toolbar-block toolbar-block--repo">
        <el-select
          v-model="repoModel"
          class="repo-input"
          placeholder="输入 owner/repo 或 GitHub 仓库地址"
          clearable
          filterable
          allow-create
          default-first-option
          :reserve-keyword="false"
          @keyup.enter="$emit('load')"
        >
          <template #prefix>
            <SmartIcon name="github" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
          </template>
          <el-option
            v-for="option in repoOptions"
            :key="option"
            :label="option"
            :value="option"
          />
        </el-select>
      </div>

      <div class="toolbar-block toolbar-block--actions">
        <el-button type="primary" class="load-btn" :loading="loading" @click="$emit('load')">加载 APK 列表</el-button>
        <el-button :disabled="loading" @click="$emit('refresh')">刷新</el-button>
        <div class="stats-group page-stats page-stats--toolbar">
          <div class="stat-item page-stat">
            <SmartIcon name="package" color="var(--color-info)" :size="14" :show-background="false" :show-decoration="false" />
            <span class="stat-label">当前 APK</span>
          </div>
          <div class="stat-item blue page-stat">
            <span class="stat-value page-stat-value">{{ total }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="toolbar-row toolbar-row--secondary">
      <div class="toolbar-block token-strip">
        <div class="token-strip__lead">
          <span class="token-strip__icon">
            <SmartIcon name="key" color="var(--color-primary)" :size="13" :show-background="false" :show-decoration="false" />
          </span>
          <div class="token-strip__copy">
            <span class="token-strip__title">GitHub API 提额</span>
            <span class="token-strip__desc">可选填写 GitHub Access Token，用于提升 API 配额并缓解匿名访问频率限制</span>
          </div>
        </div>

        <div class="token-strip__controls">
          <el-input
            v-model="tokenModel"
            type="password"
            show-password
            clearable
            class="token-input token-input--strip"
            placeholder="可选填写 GitHub Access Token"
          >
            <template #prefix>
              <SmartIcon name="key" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
            </template>
          </el-input>

          <el-button text class="token-guide-btn" @click="showTokenGuide = true">如何获取？</el-button>
        </div>
      </div>

      <div class="toolbar-block toolbar-block--meta">
        <el-input v-model="searchModel" placeholder="搜索版本、文件名或标签" clearable class="page-search github-search">
          <template #prefix>
            <SmartIcon name="search" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
          </template>
        </el-input>
      </div>
    </div>
  </div>

  <el-dialog
    v-model="showTokenGuide"
    title="GitHub Access Token 获取指南"
    width="560px"
    class="app-dialog-shell token-guide-dialog"
    :close-on-click-modal="true"
  >
    <div class="token-guide-content">
      <div class="token-guide-section">
        <div class="token-guide-title">什么时候需要 Access Token</div>
        <p class="token-guide-desc">
          当 GitHub APK 页面提示匿名访问频率已达上限时，可以填写个人 GitHub Access Token，
          用来提升 API 配额，并更稳定地读取 Releases 列表。
        </p>
      </div>

      <div class="token-guide-section">
        <div class="token-guide-title">获取步骤</div>
        <ol class="token-guide-steps">
          <li>登录 GitHub 账号，并确认账号已绑定有效邮箱。</li>
          <li>点击右上角头像，进入 `Settings`。</li>
          <li>在左侧依次进入 `Developer settings` → `Personal access tokens` → `Fine-grained tokens`。</li>
          <li>点击 `Generate new token`，填写 `Token name`，并设置建议的过期时间，例如 7 天或 30 天。</li>
          <li>按需选择 `Resource owner` 与 `Repository access`，公开仓库场景通常可选择仅授权目标仓库。</li>
          <li>在 `Permissions` 中按需勾选权限，遵循最小权限原则。</li>
          <li>点击 `Generate` 后立即复制并保存 Token，因为完整 Token 只会显示一次。</li>
          <li>将复制好的 Token 粘贴到当前输入框即可。</li>
        </ol>
      </div>

      <div class="token-guide-section">
        <div class="token-guide-title">建议权限</div>
        <p class="token-guide-desc">
          推荐优先使用 `Fine-grained token`，并遵循最小权限原则。公开仓库读取场景通常可先尝试
          `Metadata: Read-only` 与 `Contents: Read-only`；如果你的仓库或组织策略有额外要求，再按 GitHub 提示补充权限。
        </p>
      </div>

      <div class="token-guide-actions">
        <el-button type="primary" @click="handleOpenTokenPage">打开 GitHub Access Token 页面</el-button>
      </div>
    </div>
  </el-dialog>
</template>

<script setup>
import { computed, ref } from 'vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ElMessage } from 'element-plus';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  repo: { type: String, default: '' },
  repoOptions: { type: Array, default: () => [] },
  githubToken: { type: String, default: '' },
  searchQuery: { type: String, default: '' },
  total: { type: Number, default: 0 },
  loading: { type: Boolean, default: false },
});

const emit = defineEmits(['update:repo', 'update:github-token', 'update:search-query', 'load', 'refresh']);

const showTokenGuide = ref(false);

const repoModel = computed({
  get: () => props.repo,
  set: (value) => emit('update:repo', value),
});

const tokenModel = computed({
  get: () => props.githubToken,
  set: (value) => emit('update:github-token', value),
});

const searchModel = computed({
  get: () => props.searchQuery,
  set: (value) => emit('update:search-query', value),
});

async function handleOpenTokenPage() {
  try {
    await openUrl('https://github.com/settings/personal-access-tokens');
  } catch (error) {
    ElMessage.error('打开 GitHub Token 页面失败，请稍后重试');
  }
}
</script>

<style lang="scss" scoped>
.github-apk-toolbar {
  flex-direction: column;
  align-items: stretch;
  gap: 14px;
}

.toolbar-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  flex-wrap: wrap;
}

.toolbar-block {
  min-width: 0;
}

.toolbar-block--repo {
  flex: 1;
}

.toolbar-block--actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 0 0 auto;
}

.toolbar-block--meta {
  flex: 1 1 340px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  flex-wrap: wrap;
}

.repo-input {
  width: 100%;

  :deep(.el-select__wrapper) {
    border-radius: var(--radius-lg);
  }
}

.load-btn {
  min-width: 136px;
}

.page-stats--toolbar {
  margin-left: 2px;
}

.token-strip {
  flex: 1 1 620px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 42px;
  padding: 8px 12px;
  border-radius: 14px;
  border: 1px solid rgba(var(--color-primary-rgb), 0.08);
  background:
    linear-gradient(90deg, rgba(var(--color-primary-rgb), 0.035), rgba(var(--color-primary-rgb), 0.015)),
    var(--surface-panel);
}

.token-strip__lead {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.token-strip__icon {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 999px;
  background: rgba(var(--color-primary-rgb), 0.08);
}

.token-strip__copy {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
  flex-wrap: wrap;
}

.token-strip__title {
  font-size: 12px;
  font-weight: 700;
  color: var(--color-text-primary);
  white-space: nowrap;
}

.token-strip__desc {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.token-strip__controls {
  flex: 1 1 320px;
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
}

.token-input {
  flex: 1 1 280px;
  max-width: 360px;
}

.token-input--strip {
  max-width: 320px;
}

.token-input--strip :deep(.el-input__wrapper) {
  min-height: 34px;
  border-radius: 999px;
  background: var(--surface-elevated) !important;
  box-shadow:
    inset 0 0 0 1px rgba(var(--color-primary-rgb), 0.06),
    0 6px 16px -14px rgba(15, 23, 42, 0.3) !important;
}

.token-input--strip :deep(.el-input__wrapper:hover),
.token-input--strip :deep(.el-input__wrapper.is-focus) {
  background: var(--surface-elevated-strong) !important;
  box-shadow:
    inset 0 0 0 1px rgba(var(--color-primary-rgb), 0.16),
    0 8px 18px -16px rgba(var(--color-primary-rgb), 0.32) !important;
}

.token-guide-btn {
  flex: 0 0 auto;
  min-height: 30px;
  padding-inline: 6px;
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 600;
}

.github-search {
  flex: 1;
  width: auto;
}

.token-guide-dialog {
  .el-dialog__body {
    padding-top: 8px;
  }
}

.token-guide-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.token-guide-section {
  padding-bottom: 14px;
  border-bottom: 1px solid var(--color-divider);

  &:last-of-type {
    padding-bottom: 0;
    border-bottom: none;
  }
}

.token-guide-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 8px;
}

.token-guide-desc {
  margin: 0;
  line-height: 1.8;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.token-guide-steps {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  color: var(--color-text-secondary);
  line-height: 1.75;
  font-size: 13px;
}

.token-guide-actions {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 1320px) {
  .toolbar-block--meta {
    justify-content: flex-start;
  }
}

@media (max-width: 980px) {
  .token-strip {
    align-items: flex-start;
    flex-direction: column;
  }

  .token-strip__lead {
    align-items: flex-start;
  }

  .token-strip__copy {
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
  }

  .token-strip__controls {
    width: 100%;
    justify-content: flex-start;
  }

  .token-input,
  .github-search {
    max-width: none;
  }
}

@media (max-width: 760px) {
  .toolbar-block--actions,
  .toolbar-block--meta {
    width: 100%;
  }

  .toolbar-block--actions {
    justify-content: flex-start;
  }

  .toolbar-block--meta {
    justify-content: flex-start;
  }
}
</style>
