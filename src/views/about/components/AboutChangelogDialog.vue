<template>
  <el-dialog
    v-model="dialogVisible"
    title="更新日志历史"
    width="600px"
    class="app-dialog-shell changelog-dialog"
    :append-to-body="true"
    align-center
  >
    <div class="changelog-container">
      <div v-for="item in changelog" :key="item.version" class="changelog-item">
        <div class="changelog-header">
          <span class="version-tag">v{{ item.version }}</span>
          <span class="release-date">{{ item.date }}</span>
        </div>
        <ul class="notes-list">
          <li v-for="(note, index) in item.notes" :key="index">{{ note }}</li>
        </ul>
      </div>
      <el-empty v-if="changelog.length === 0" description="暂无记录" />
    </div>
  </el-dialog>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  visible: { type: Boolean, default: false },
  changelog: { type: Array, default: () => [] },
});

const emit = defineEmits(['update:visible']);

const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value),
});
</script>

<style lang="scss">
.changelog-dialog {
  --app-dialog-radius: 16px;

  .el-dialog__header {
    padding: 20px 25px;
    border-bottom: 1px solid var(--color-border);
  }

  .el-dialog__title {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .el-dialog__body {
    padding: 0;
  }

  .changelog-container {
    max-height: 450px;
    overflow-y: auto;
    padding: 25px;

    &::-webkit-scrollbar {
      width: 6px;
    }

    &::-webkit-scrollbar-thumb {
      background: var(--scrollbar-thumb);
      border-radius: 10px;
    }
  }

  .changelog-item {
    position: relative;
    padding-bottom: 25px;
    text-align: left;

    &:not(:last-child)::after {
      content: '';
      position: absolute;
      left: 15px;
      top: 30px;
      bottom: 0;
      width: 2px;
      background: linear-gradient(to bottom, rgba(var(--color-success-rgb), 0.65) 0%, transparent 100%);
      opacity: 0.3;
    }
  }

  .changelog-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .version-tag {
    background: var(--success-gradient);
    color: var(--text-on-primary);
    padding: 2px 10px;
    border-radius: 20px;
    font-weight: 600;
    font-size: 13px;
    box-shadow: var(--success-shadow-soft);
  }

  .release-date {
    font-size: 13px;
    color: var(--color-text-secondary);
    font-family: 'JetBrains Mono', monospace;
  }

  .notes-list {
    margin: 0;
    padding-left: 20px;

    li {
      font-size: 14px;
      color: var(--color-text-secondary);
      margin-bottom: 6px;
      line-height: 1.6;
      list-style-type: disc;

      &::marker {
        color: var(--color-success);
      }
    }
  }
}
</style>
