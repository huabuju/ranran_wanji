<template>
  <div class="top-action-bar page-toolbar surface-card">
    <div class="left-actions page-toolbar-left">
      <div class="breadcrumb">
        <span class="crumb" :class="{ clickable: currentLevel !== 'brands' }" @click="$emit('go-level', 'brands')">全部</span>

        <template v-if="currentLevel !== 'brands'">
          <span class="crumb-sep">/</span>
          <span class="crumb" :class="{ clickable: currentLevel === 'roms' }" @click="$emit('go-level', 'models')">{{ selectedBrand?.name }}</span>
        </template>

        <template v-if="currentLevel === 'roms'">
          <span class="crumb-sep">/</span>
          <span class="crumb active">{{ selectedModel?.name }}</span>
        </template>
      </div>

      <div v-if="currentLevel === 'brands'" class="source-switch">
        <el-segmented
          v-model="sourceKeyModel"
          class="source-switch__segmented"
          :options="sourceSegmentOptions"
          size="default"
        />
      </div>
    </div>

    <div class="right-actions page-toolbar-right">
      <template v-if="currentLevel === 'brands'">
        <div class="stats-group page-stats summary-stats">
          <SmartIcon name="device" color="var(--color-primary)" :size="14" :show-background="false" :show-decoration="false" />
          <span class="stat-label">共</span>
          <span class="stat-value stat-value-primary">{{ totalModelCount }}</span>
          <span class="stat-label">个机型</span>
        </div>
      </template>

      <template v-else-if="currentLevel === 'models'">
        <el-input v-model="modelSearchModel" placeholder="搜索机型..." clearable class="rom-search-input page-search page-search--narrow">
          <template #prefix>
            <SmartIcon name="search" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
          </template>
        </el-input>
        <div class="stats-group page-stats summary-stats">
          <SmartIcon name="device" color="var(--color-primary)" :size="14" :show-background="false" :show-decoration="false" />
          <span class="stat-label">共</span>
          <span class="stat-value stat-value-primary">{{ modelCount }}</span>
          <span class="stat-label">个机型</span>
        </div>
      </template>

      <template v-else-if="currentLevel === 'roms'">
        <el-input v-model="romSearchModel" placeholder="搜索版本号..." clearable class="rom-search-input page-search page-search--narrow">
          <template #prefix>
            <SmartIcon name="search" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
          </template>
        </el-input>
        <div class="view-toggle">
          <button class="toggle-btn" :class="{ active: viewMode === 'grid' }" title="网格视图" @click="$emit('update:view-mode', 'grid')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" />
            </svg>
          </button>
          <button class="toggle-btn" :class="{ active: viewMode === 'list' }" title="列表视图" @click="$emit('update:view-mode', 'list')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" />
            </svg>
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  currentLevel: { type: String, default: 'brands' },
  selectedBrand: { type: Object, default: null },
  selectedModel: { type: Object, default: null },
  modelSearchRaw: { type: String, default: '' },
  romSearchRaw: { type: String, default: '' },
  totalModelCount: { type: Number, default: 0 },
  totalRomCount: { type: Number, default: 0 },
  modelCount: { type: Number, default: 0 },
  modelRomCount: { type: Number, default: 0 },
  viewMode: { type: String, default: 'grid' },
  sourceKey: { type: String, default: 'xiaomirom' },
  sourceOptions: { type: Array, default: () => [] },
});

const emit = defineEmits(['go-level', 'update:model-search-raw', 'update:rom-search-raw', 'update:view-mode', 'update:source-key']);

const modelSearchModel = computed({
  get: () => props.modelSearchRaw,
  set: (value) => emit('update:model-search-raw', value),
});

const romSearchModel = computed({
  get: () => props.romSearchRaw,
  set: (value) => emit('update:rom-search-raw', value),
});

const sourceKeyModel = computed({
  get: () => props.sourceKey,
  set: (value) => emit('update:source-key', value),
});

const sourceSegmentOptions = computed(() => (
  (Array.isArray(props.sourceOptions) ? props.sourceOptions : []).map((item) => ({
    label: item.label,
    value: item.key,
  }))
));
</script>

<style lang="scss" scoped>
.top-action-bar { flex-shrink: 0; min-height: 52px; }

.left-actions {
  flex: 1;
  min-width: 0;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;

  .crumb {
    color: var(--color-text-muted);
    transition: color 0.15s;
    white-space: nowrap;

    &.clickable {
      cursor: pointer;

      &:hover {
        color: var(--color-primary);
      }
    }

    &.active {
      color: var(--color-text-primary);
      font-weight: 600;
    }
  }

  .crumb-sep {
    color: var(--color-border);
    font-size: 16px;
    line-height: 1;
  }
}

.right-actions {
  display: flex;
  align-items: center;
  gap: 12px;

  .stats-group {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 12px;
    background: var(--bg-hover-subtle);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    font-size: 12px;
    color: var(--color-text-secondary);

    .stat-value.cyan {
      font-weight: 700;
      color: var(--color-info);
    }
  }

  .summary-stats {
    min-height: 30px;
    padding: 4px 14px;
  }
}

.stat-value-primary {
  color: var(--color-primary);
  font-weight: 700;
}

.view-toggle {
  display: flex;
  flex-shrink: 0;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;

  .toggle-btn {
    width: 32px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: var(--surface-soft);
    cursor: pointer;
    color: var(--color-text-muted);
    transition: all 0.2s;

    svg { width: 15px; height: 15px; }

    &:hover {
      background: var(--bg-card-hover);
      color: var(--color-text-primary);
    }

    &.active {
      background: var(--color-primary);
      color: var(--text-on-primary);
    }

    & + .toggle-btn {
      border-left: 1px solid var(--color-divider);
    }
  }
}

.source-switch {
  display: flex;
  align-items: center;
  min-height: 34px;
  padding: 0;
  background: var(--bg-hover-subtle);
  border: 1px solid var(--color-border);
  border-radius: 20px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  overflow: hidden;
}

.source-switch__segmented {
  --el-segmented-bg-color: transparent;
  --el-segmented-padding: 0;
  --el-segmented-item-selected-bg-color: var(--color-primary);
  --el-segmented-item-selected-color: #ffffff;
  --el-segmented-item-hover-bg-color: rgba(var(--color-primary-rgb), 0.08);
  display: flex;
  align-self: stretch;
  min-height: 34px;
  width: 100%;
  padding: 0;
  border: none;
  border-radius: inherit;
  overflow: hidden;
}

:deep(.source-switch__segmented .el-segmented__group) {
  min-height: 34px;
}

:deep(.source-switch__segmented .el-segmented__item-selected) {
  border-radius: inherit;
}

:deep(.source-switch__segmented .el-segmented__item) {
  min-width: 88px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-primary);
  border-radius: 14px;
  opacity: 1;
  transition: color 0.2s ease, background-color 0.2s ease, box-shadow 0.2s ease;
}

:deep(.source-switch__segmented .el-segmented__item:hover) {
  color: var(--color-primary);
}

:deep(.source-switch__segmented .el-segmented__item.is-selected) {
  color: #ffffff;
  box-shadow: 0 8px 18px rgba(var(--color-primary-rgb), 0.18);
  text-shadow: 0 1px 2px rgba(15, 23, 42, 0.16);
}

:deep(.source-switch__segmented .el-segmented__item.is-selected:hover) {
  color: #ffffff;
}
</style>
