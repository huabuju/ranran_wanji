<template>
  <div class="top-action-bar page-toolbar surface-card">
    <div class="left-actions page-toolbar-left">
      <div class="input-wrapper">
        <el-input v-model="pathModel" placeholder="请选择本地文件/输入在线 URL 地址" clearable class="page-search" :disabled="extracting">
          <template #prefix>
            <SmartIcon name="package" color="var(--color-text-muted)" :size="14" :show-background="false" :show-decoration="false" />
          </template>
        </el-input>
      </div>
      <el-button type="primary" :disabled="extracting" @click="$emit('select-file')">选择文件</el-button>
    </div>

    <div class="right-actions page-toolbar-right">
      <div class="stats-group page-stats">
        <div class="stat-item page-stat">
          <SmartIcon name="partition" color="var(--color-info)" :size="14" :show-background="false" :show-decoration="false" />
          <span class="stat-label">已发现分区</span>
        </div>
        <div class="stat-item blue page-stat">
          <span class="stat-value page-stat-value">{{ total }}</span>
        </div>
      </div>
      <el-button class="help-btn" circle size="small" @click="$emit('show-help')">
        <span class="help-btn-icon">?</span>
      </el-button>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  payloadPath: { type: String, default: '' },
  extracting: { type: Boolean, default: false },
  total: { type: Number, default: 0 },
});

const emit = defineEmits(['update:payload-path', 'select-file', 'show-help']);

const pathModel = computed({
  get: () => props.payloadPath,
  set: (value) => emit('update:payload-path', value),
});
</script>

<style lang="scss" scoped>
.input-wrapper {
  width: 460px;
  max-width: 100%;
}

.help-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  border-radius: 50%;
  border: 1.5px solid rgba(var(--color-info-rgb), 0.24);
  background: rgba(var(--color-info-rgb), 0.12);
  color: var(--color-info);
  transition: all 0.2s;
  flex-shrink: 0;

  &:hover {
    background: var(--color-info);
    border-color: var(--color-info);
    color: var(--text-on-primary);
    box-shadow: 0 2px 8px rgba(var(--color-info-rgb), 0.35);
  }
}

.help-btn-icon {
  font-size: 13px;
  font-weight: 700;
  line-height: 1;
  font-family: serif;
}

.right-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
</style>
