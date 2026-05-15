<template>
  <div class="ctrl-card">
    <div class="ctrl-card__header">
      <SmartIcon name="display" color="var(--color-info)" :size="16" :show-background="false" :show-decoration="false" />
      <span class="ctrl-title">显示属性</span>
    </div>
    <div class="ctrl-card__body">
      <div v-if="display.curRes || display.curDensity" class="info-strip">
        <span v-if="display.curRes">分辨率：<b>{{ display.curRes }}</b></span>
        <span v-if="display.curDensity">密度：<b>{{ display.curDensity }} dpi</b></span>
      </div>

      <div class="field-group">
        <div class="field-label">屏幕尺寸</div>
        <div class="row-inputs">
          <el-input v-model="display.width" placeholder="横向" :disabled="!isConnected" />
          <span class="dim-sep">×</span>
          <el-input v-model="display.height" placeholder="纵向" :disabled="!isConnected" />
        </div>
      </div>

      <div class="field-group">
        <div class="field-label">显示密度 (DPI)</div>
        <div class="row-inputs">
          <el-input v-model="display.density" placeholder="如：480" :disabled="!isConnected" class="flex-grow-input" />
        </div>
      </div>

      <div class="action-row">
        <el-button type="primary" class="flex-grow-button" :loading="loading" :disabled="!isConnected" @click="$emit('apply-display')">修改</el-button>
        <el-button class="flex-grow-button" :disabled="!isConnected" @click="$emit('reset-display')">恢复默认</el-button>
      </div>

      <div class="divider section-divider" />

      <div class="field-group">
        <div class="field-label brightness-label">
          <span>屏幕亮度</span>
          <div class="auto-toggle">
            <span class="auto-text">自动亮度</span>
            <el-switch
              v-model="brightness.auto"
              :disabled="!isConnected"
              size="small"
              @change="(value) => $emit('set-auto-brightness', value)"
            />
          </div>
        </div>
        <div class="brightness-slider-row">
          <SmartIcon name="brightness" color="var(--color-warning)" :size="14" :show-background="false" :show-decoration="false" />
          <el-slider
            v-model="brightness.value"
            class="brightness-slider"
            :max="255"
            :disabled="!isConnected"
            :show-tooltip="false"
            @change="(value) => $emit('set-brightness', value)"
          />
          <span class="brightness-val">{{ Math.round(brightness.value / 255 * 100) }}%</span>
        </div>
      </div>

      <div class="divider" />

      <div class="misc-switch-row">
        <div class="misc-switch-info">
          <span class="misc-switch-desc">在状态栏时钟后显示秒数</span>
        </div>
        <el-switch
          v-model="statusBar.clockSeconds"
          :disabled="!isConnected"
          size="small"
          @change="(value) => $emit('toggle-clock-seconds', value)"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import SmartIcon from '@/components/common/SmartIcon.vue';

defineProps({
  display: {
    type: Object,
    required: true,
  },
  brightness: {
    type: Object,
    required: true,
  },
  statusBar: {
    type: Object,
    required: true,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  isConnected: {
    type: Boolean,
    default: false,
  },
});

defineEmits(['apply-display', 'reset-display', 'set-auto-brightness', 'set-brightness', 'toggle-clock-seconds']);
</script>

<style lang="scss" scoped>
@use './shared.scss';

.brightness-label {
  display: flex;
  align-items: center;
  justify-content: space-between;

  .auto-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .auto-text {
    font-size: 11px;
    color: var(--color-text-muted);
  }
}

.brightness-slider-row {
  display: flex;
  align-items: center;
  gap: 10px;

  .brightness-slider {
    flex: 1;
  }

  .brightness-val {
    width: 36px;
    text-align: right;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }
}

.misc-switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;

  .misc-switch-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .misc-switch-desc {
    font-size: 11px;
    color: var(--color-text-muted);
  }
}
</style>
