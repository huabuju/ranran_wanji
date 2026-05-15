<template>
  <div
    class="command-card"
    :class="{
      'is-batch-mode': isBatchMode,
      'is-selected': isSelected,
      'is-disabled': isBatchMode && item.isBuiltIn,
      'is-selectable': isBatchMode && !item.isBuiltIn,
    }"
    @click="$emit('click', item)"
  >
    <div v-if="isBatchMode" class="batch-selector" :class="{ checked: isSelected, disabled: item.isBuiltIn }">
      <span v-if="item.isBuiltIn">内置</span>
      <span v-else>{{ isSelected ? '已选' : '选择' }}</span>
    </div>

    <div class="cmd-version">{{ item.name }}</div>

    <div class="cmd-tags">
      <span class="android-tag small" :class="item.isBuiltIn ? 'tag-a13' : 'tag-a15'">
        {{ item.isBuiltIn ? '内置指令' : '自定义' }}
      </span>
      <span class="android-tag small tag-card">
        {{ typeLabel }}
      </span>
    </div>

    <div class="code-snippet-box">
      <span class="cmd-prefix">$</span>
      {{ item.command }}
    </div>

    <div class="cmd-actions">
      <template v-if="!isBatchMode">
        <template v-if="!item.isBuiltIn">
          <button class="dl-btn" title="编辑指令" @click.stop="$emit('edit', item)">
            <el-icon><EditPen /></el-icon>
          </button>
          <el-popconfirm title="确定要删除该指令吗？" @confirm="$emit('delete', item.id)">
            <template #reference>
              <button class="dl-btn" title="删除指令" @click.stop>
                <el-icon><Delete /></el-icon>
              </button>
            </template>
          </el-popconfirm>
        </template>
        <button class="dl-btn primary" title="运行指令" @click.stop="$emit('execute', item)">
          <span>运行指令</span>
        </button>
      </template>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { Delete, EditPen } from '@element-plus/icons-vue';

const props = defineProps({
  item: {
    type: Object,
    required: true,
  },
  isBatchMode: {
    type: Boolean,
    default: false,
  },
  isSelected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['click', 'edit', 'delete', 'execute']);

const typeLabel = computed(() => {
  const command = props.item.command || '';
  if (command.startsWith('adb shell')) return 'ADB Shell';
  if (command.startsWith('fastboot')) return 'Fastboot';
  if (command.startsWith('adb')) return 'ADB';
  return 'Unknown';
});
</script>

<style lang="scss" scoped>
@use './shared.scss';

.batch-selector {
  position: absolute;
  top: 12px;
  right: 12px;
  min-width: 50px;
  text-align: center;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  color: var(--color-text-secondary);
  background: var(--surface-soft);
  border: 1px solid var(--color-border);

  &.checked {
    background: rgba(var(--color-danger-rgb), 0.12);
    color: var(--color-danger);
    border-color: rgba(var(--color-danger-rgb), 0.3);
  }

  &.disabled {
    background: var(--surface-panel);
    color: var(--color-text-muted);
    border-color: var(--color-border);
  }
}

.cmd-version {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cmd-tags {
  display: flex;
  gap: 6px;
}

.android-tag {
  flex: 1;
  display: inline-block;
  align-items: center;
  border-radius: 20px;
  font-weight: 600;
  text-align: center;

  &.small {
    padding: 2px 8px;
    font-size: 11px;
  }

  &.tag-a13 {
    background: var(--tag-a13-bg);
    color: var(--tag-a13-text);
    border: 1px solid var(--tag-a13-border);
  }

  &.tag-a15 {
    background: var(--tag-a15-bg);
    color: var(--tag-a15-text);
    border: 1px solid var(--tag-a15-border);
  }

  &.tag-card {
    background: var(--tag-card-bg);
    color: var(--tag-card-text);
    border: 1px solid var(--tag-card-border);
  }
}

.code-snippet-box {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 11px;
  color: var(--color-text-secondary);
  background: var(--surface-soft);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  white-space: nowrap;
  text-overflow: ellipsis;
  line-height: 1.4;
  border: 1px dashed var(--color-border);
  overflow: hidden;

  .cmd-prefix {
    color: var(--color-text-muted);
    margin-right: 4px;
  }
}

.cmd-actions {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.dl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 32px;
  width: 32px;
  border: 1px solid var(--color-border);
  background: var(--surface-soft);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-secondary);

  &:hover {
    background: var(--surface-strong);
    border-color: rgba(var(--color-primary-rgb), 0.18);
    color: var(--color-primary);
  }

  &.primary {
    flex: 1;
    width: auto;
    background: var(--color-primary-light);
    border-color: var(--color-primary-light);
    color: var(--color-primary);
    gap: 6px;

    span {
      font-size: 12px;
      font-weight: 600;
    }

    &:hover {
      background: rgba(var(--color-primary-rgb), 0.16);
      border-color: rgba(var(--color-primary-rgb), 0.16);
    }
  }
}
</style>
