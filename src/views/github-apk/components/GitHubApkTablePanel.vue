<template>
  <div class="left-table-panel page-table-card surface-card-strong">
    <div class="table-container page-table-inner">
      <el-auto-resizer>
        <template #default="{ height, width }">
          <el-table-v2
            :columns="columns"
            :data="data"
            :width="width"
            :height="height"
            fixed
            class="custom-table-v2"
          >
            <template #cell="{ rowData: row, column }">
              <template v-if="column.key === 'release'">
                <div class="release-cell">
                  <span class="release-tag">{{ row.releaseTag || '-' }}</span>
                  <span class="release-name">{{ row.releaseName || row.assetName }}</span>
                </div>
              </template>

              <template v-else-if="column.key === 'asset'">
                <div class="asset-cell">
                  <el-tooltip :content="row.assetName" placement="top" :show-after="300">
                    <span class="asset-name">{{ row.assetName }}</span>
                  </el-tooltip>
                </div>
              </template>

              <template v-else-if="column.key === 'channel'">
                <span class="channel-badge" :class="row.channelClass">{{ row.channelLabel }}</span>
              </template>

              <template v-else-if="column.key === 'size'">
                <span class="mono-text">{{ formatSize(row.size) }}</span>
              </template>

              <template v-else-if="column.key === 'downloads'">
                <span class="mono-text">{{ formatCount(row.downloadCount) }}</span>
              </template>

              <template v-else-if="column.key === 'publishedAt'">
                <span class="date-text">{{ formatDate(row.publishedAt) }}</span>
              </template>

              <template v-else-if="column.key === 'actions'">
                <div class="action-cell">
                  <el-button size="small" type="primary" plain @click.stop="$emit('download', row)">下载</el-button>
                  <el-button size="small" @click.stop="$emit('copy-url', row)">复制地址</el-button>
                </div>
              </template>
            </template>

            <template #empty>
              <div class="empty-state">
                <SmartIcon name="github" :size="32" color="var(--color-text-muted)" class="mb-2" />
                <span>{{ loading ? '正在读取 GitHub 发行版...' : '暂无 APK 数据' }}</span>
              </div>
            </template>
          </el-table-v2>
        </template>
      </el-auto-resizer>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  columns: { type: Array, default: () => [] },
  data: { type: Array, default: () => [] },
  loading: { type: Boolean, default: false },
  formatSize: { type: Function, required: true },
  formatCount: { type: Function, required: true },
  formatDate: { type: Function, required: true },
});

defineEmits(['download', 'copy-url']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.left-table-panel {
  flex: 1;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}

.table-container {
  flex: 1;
  overflow: hidden;
  height: 100%;
  position: relative;

  :deep(.custom-table-v2) {
    background: transparent;
    border: none;

    .el-table-v2__header-cell {
      background: var(--table-header-bg);
      color: var(--color-text-secondary);
      font-size: 12px;
      font-weight: 600;
      border-bottom: 1px solid var(--color-divider);
      padding: 0 12px;
      display: flex;
      align-items: center;
    }

    .el-table-v2__row-cell {
      border-bottom: 1px solid var(--color-divider);
      background: transparent;
      display: flex;
      align-items: center;
      padding: 0 12px;
    }

    .el-table-v2__row:hover .el-table-v2__row-cell {
      background-color: var(--table-row-hover);
    }
  }
}

.release-cell,
.asset-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.release-tag,
.mono-text {
  font-size: 11px;
  font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace;
  color: var(--color-text-secondary);
}

.release-name,
.asset-name {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.channel-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56px;
  height: 26px;
  padding: 0 10px;
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 700;
  border: 1px solid transparent;

  &.is-stable {
    color: var(--color-success);
    background: rgba(var(--color-success-rgb), 0.1);
    border-color: rgba(var(--color-success-rgb), 0.18);
  }

  &.is-prerelease {
    color: var(--color-warning);
    background: rgba(var(--color-warning-rgb), 0.12);
    border-color: rgba(var(--color-warning-rgb), 0.2);
  }

  &.is-draft {
    color: var(--color-text-secondary);
    background: var(--surface-soft);
    border-color: var(--color-border);
  }
}

.date-text {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.action-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--color-text-muted);
}
</style>
