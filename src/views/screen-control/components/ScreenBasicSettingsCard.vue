<template>
  <el-card class="custom-card" shadow="hover">
    <template #header>
      <div class="card-header flex-y-center">
        <SmartIcon name="settings" color="var(--color-primary)" :size="18" :show-background="false" :show-decoration="false" />
        <span>基础设置</span>
      </div>
    </template>

    <div v-for="item in sliderItems" :key="item.key" class="form-group">
      <label>{{ item.label }}</label>
      <div class="slider-wrapper">
        <el-slider v-model="form[item.key]" :step="1" :min="0" :max="3" :marks="item.marks" :show-tooltip="false" :disabled="isStreaming" />
      </div>
    </div>
  </el-card>
</template>

<script setup>
import { computed } from 'vue';
import SmartIcon from '@/components/common/SmartIcon.vue';

const props = defineProps({
  form: { type: Object, required: true },
  isStreaming: { type: Boolean, default: false },
  resMarks: { type: Object, required: true },
  fpsMarks: { type: Object, required: true },
  bitrateMarks: { type: Object, required: true },
});

const sliderItems = computed(() => [
  { key: 'maxSizeIndex', label: '分辨率选项 (Max Size)', marks: props.resMarks },
  { key: 'maxFpsIndex', label: '最大帧率 (Max FPS)', marks: props.fpsMarks },
  { key: 'bitRateIndex', label: '视频比特率 (Bitrate)', marks: props.bitrateMarks },
]);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.form-group {
  margin-bottom: 36px;

  &:last-child {
    margin-bottom: 16px;
  }

  label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
    margin-bottom: 8px;
  }
}

.slider-wrapper {
  padding: 0 16px;
  margin-top: -4px;

  :deep(.el-slider__marks-text) {
    font-size: 12px;
    color: var(--color-text-muted);
    margin-top: 14px;
    white-space: nowrap;
    font-weight: 500;
  }

  :deep(.el-slider__stop) {
    background-color: var(--color-border);
  }
}
</style>
