<template>
  <GlassModal :show="show" max-width="560px" click-to-close @close="$emit('close')">
    <div class="url-choice-dialog">
      <div class="dialog-header">
        <div>
          <div class="dialog-eyebrow">多地址可用</div>
          <h3>{{ title }}</h3>
          <p v-if="description">{{ description }}</p>
        </div>
        <button class="close-btn" type="button" aria-label="关闭" @click="$emit('close')">×</button>
      </div>

      <div class="url-options">
        <button
          v-for="(url, index) in urls"
          :key="`${url}-${index}`"
          class="url-option"
          type="button"
          @click="$emit('select', url)"
        >
          <span class="option-index">{{ index + 1 }}</span>
          <span class="option-main">
            <span class="option-url">{{ url }}</span>
          </span>
          <span class="option-action">{{ actionLabel }}</span>
        </button>
      </div>

      <div class="dialog-footer">
        <button class="cancel-btn" type="button" @click="$emit('close')">取消</button>
      </div>
    </div>
  </GlassModal>
</template>

<script setup>
import GlassModal from '@/components/common/GlassModal.vue';

defineProps({
  show: { type: Boolean, default: false },
  title: { type: String, default: '选择下载地址' },
  description: { type: String, default: '' },
  urls: { type: Array, default: () => [] },
  actionLabel: { type: String, default: '选择' },
});

defineEmits(['close', 'select']);

</script>

<style lang="scss" scoped>
.url-choice-dialog {
  padding: 22px;
  background:
    radial-gradient(circle at 16% 8%, rgba(var(--color-primary-rgb), 0.12), transparent 28%),
    var(--surface-elevated-strong);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 18px;

  h3 {
    margin: 4px 0 6px;
    color: var(--color-text-primary);
    font-size: 18px;
    line-height: 1.35;
  }

  p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 13px;
    line-height: 1.7;
  }
}

.dialog-eyebrow {
  display: inline-flex;
  align-items: center;
  height: 24px;
  padding: 0 10px;
  border: 1px solid rgba(var(--color-primary-rgb), 0.14);
  border-radius: var(--radius-full);
  background: rgba(var(--color-primary-rgb), 0.08);
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 700;
}

.close-btn {
  flex: 0 0 auto;
  width: 30px;
  height: 30px;
  border: 1px solid var(--color-border);
  border-radius: 50%;
  background: var(--surface-soft);
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 20px;
  line-height: 1;
  transition: all 0.2s ease;

  &:hover {
    background: var(--surface-chip-hover);
    color: var(--color-text-primary);
    transform: rotate(90deg);
  }
}

.url-options {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 320px;
  overflow-y: auto;
  padding: 3px 4px 3px 0;
}

.url-option {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 16px;
  background: var(--surface-panel);
  cursor: pointer;
  text-align: left;
  transition: all 0.2s ease;

  &:hover {
    border-color: rgba(var(--color-primary-rgb), 0.34);
    background: var(--surface-elevated-strong);
    box-shadow: var(--primary-shadow-subtle);

    .option-action {
      background: var(--color-primary);
      color: var(--text-on-primary);
    }
  }
}

.option-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 11px;
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-size: 13px;
  font-weight: 800;
}

.option-main {
  min-width: 0;
  display: flex;
  align-items: center;
}

.option-url {
  color: var(--color-text-secondary);
  font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.55;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.option-action {
  padding: 7px 10px;
  border-radius: 999px;
  background: var(--surface-soft);
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 700;
  transition: all 0.2s ease;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}

.cancel-btn {
  height: 34px;
  padding: 0 16px;
  border: 1px solid var(--color-border);
  border-radius: 999px;
  background: var(--surface-soft);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 13px;
  font-weight: 700;
  transition: all 0.2s ease;

  &:hover {
    background: var(--surface-chip-hover);
    color: var(--color-text-primary);
  }
}
</style>
