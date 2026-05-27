<template>
  <template v-if="viewMode === 'grid'">
    <div class="rom-grid">
      <div v-for="rom in roms" :key="rom.key" class="rom-card">
        <div class="rom-version">{{ rom.version }}</div>
        <div v-if="rom.name" class="rom-name">{{ rom.name }}</div>
        <div class="rom-tags">
          <span v-if="rom.regionLabel" class="android-tag small tag-region">{{ rom.regionLabel }}</span>
          <span class="android-tag small" :class="getAndroidCls(rom.android)">{{ rom.androidLabel }}</span>
          <span class="android-tag small" :class="getFlashTypeCls(rom.flashType)">{{ getFlashTypeLabel(rom.flashType) }}</span>
          <span v-if="rom.mirrorCount > 1" class="android-tag small tag-mirror">{{ rom.mirrorCount }} 个镜像</span>
        </div>
        <div class="rom-actions">
          <button class="dl-btn primary copy-btn" title="复制下载地址" @click.stop="$emit('copy-url', rom)"><span>复制地址</span></button>
          <button class="dl-btn primary" title="使用下载器多线程下载" @click.stop="$emit('download', rom)"><span>下载器</span></button>
        </div>
      </div>
    </div>
  </template>

  <template v-else>
    <div class="rom-list">
      <div v-for="rom in roms" :key="rom.key" class="rom-list-item">
        <div class="list-left">
          <span v-if="rom.regionLabel" class="android-tag small tag-region">{{ rom.regionLabel }}</span>
          <span class="android-tag small" :class="getAndroidCls(rom.android)">{{ rom.androidLabel }}</span>
          <span class="android-tag small" :class="getFlashTypeCls(rom.flashType)">{{ getFlashTypeLabel(rom.flashType) }}</span>
          <span v-if="rom.mirrorCount > 1" class="android-tag small tag-mirror">{{ rom.mirrorCount }} 个镜像</span>
          <span class="list-version">{{ rom.version }}</span>
          <span v-if="rom.name" class="list-name">{{ rom.name }}</span>
        </div>
        <div class="list-actions">
          <button class="dl-btn primary copy-btn" title="复制下载地址" @click.stop="$emit('copy-url', rom)"><span>复制地址</span></button>
          <button class="dl-btn primary" title="使用下载器多线程下载" @click.stop="$emit('download', rom)"><span>下载器</span></button>
        </div>
      </div>
    </div>
  </template>
</template>

<script setup>
defineProps({
  roms: { type: Array, default: () => [] },
  viewMode: { type: String, default: 'grid' },
  getAndroidCls: { type: Function, required: true },
  getFlashTypeCls: { type: Function, required: true },
  getFlashTypeLabel: { type: Function, required: true },
});

defineEmits(['copy-url', 'download']);
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.android-tag {
  display: inline-block; align-items: center; padding: 3px 10px; border-radius: 20px; font-size: 12px; font-weight: 600;
  &.small { padding: 2px 8px; font-size: 11px; }
  &.tag-a15 { background: var(--tag-a15-bg); color: var(--tag-a15-text); border: 1px solid var(--tag-a15-border); }
  &.tag-a14 { background: var(--tag-a14-bg); color: var(--tag-a14-text); border: 1px solid var(--tag-a14-border); }
  &.tag-a13 { background: var(--tag-a13-bg); color: var(--tag-a13-text); border: 1px solid var(--tag-a13-border); }
  &.tag-legacy { background: var(--tag-legacy-bg); color: var(--tag-legacy-text); border: 1px solid var(--tag-legacy-border); }
  &.tag-region { background: rgba(99, 102, 241, 0.08); color: #4f46e5; border: 1px solid rgba(99, 102, 241, 0.22); }
  &.tag-card { background: var(--tag-card-bg); color: var(--tag-card-text); border: 1px solid var(--tag-card-border); }
  &.tag-fastboot { background: var(--tag-fastboot-bg); color: var(--tag-fastboot-text); border: 1px solid var(--tag-fastboot-border); }
  &.tag-firmware { background: var(--tag-firmware-bg); color: var(--tag-firmware-text); border: 1px solid var(--tag-firmware-border); }
  &.tag-mirror { background: var(--surface-soft); color: var(--color-text-secondary); border: 1px solid var(--color-border); }
}

.rom-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
.rom-card {
  background: var(--bg-glass); backdrop-filter: var(--blur-glass); border: 1px solid var(--color-border); border-radius: var(--radius-md);
  padding: 24px 20px; display: flex; flex-direction: column; gap: 14px; box-shadow: var(--shadow-card);
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 340ms);
  @include pageCard.overview-main-card-hoverable(var(--bg-glass));
}

.rom-card:nth-child(1) {
  --page-enter-delay: 0ms;
}

.rom-card:nth-child(2) {
  --page-enter-delay: 40ms;
}

.rom-card:nth-child(3) {
  --page-enter-delay: 80ms;
}

.rom-card:nth-child(4) {
  --page-enter-delay: 120ms;
}

.rom-card:nth-child(5) {
  --page-enter-delay: 160ms;
}

.rom-card:nth-child(6) {
  --page-enter-delay: 200ms;
}
.rom-version { font-size: 14px; font-weight: 700; color: var(--color-text-primary); font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace; }
.rom-name { font-size: 12px; color: var(--color-text-secondary); min-height: 18px; }
.rom-tags { display: flex; flex-wrap: wrap; gap: 6px; }
.rom-actions, .list-actions { display: flex; gap: 8px; }
.dl-btn {
  display: flex; align-items: center; justify-content: center; height: 32px; border: 1px solid var(--color-border); background: var(--surface-soft); border-radius: 6px; cursor: pointer; transition: all 0.2s;
  &:hover { background: var(--surface-chip-hover); border-color: var(--border-soft); }
  &.primary {
    flex: 1; background: var(--color-primary-light); border-color: rgba(var(--color-primary-rgb), 0.18); color: var(--color-primary); gap: 6px;
    span { font-size: 12px; font-weight: 600; }
    &:hover { background: var(--color-primary); color: var(--text-on-primary); box-shadow: var(--primary-shadow-soft); }
    &.copy-btn {
      background: var(--surface-soft); border-color: var(--color-border); color: var(--color-text-secondary);
      &:hover { background: var(--surface-chip-hover); border-color: var(--border-soft); color: var(--color-primary); box-shadow: none; }
    }
  }
}

.rom-list { display: flex; flex-direction: column; gap: 6px; }
.rom-list-item {
  background: var(--bg-card); border: 1px solid var(--color-divider); border-radius: var(--radius-sm); padding: 10px 16px; display: flex; align-items: center; gap: 16px;
  @include pageCard.toolkit-page-enter(var(--page-enter-delay, 0ms), 320ms);
  @include pageCard.overview-main-card-hoverable(var(--bg-card));
  .list-left { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0; flex-wrap: wrap; }
  .list-version { font-size: 13px; font-weight: 700; color: var(--color-text-primary); font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace; }
  .list-name { font-size: 12px; color: var(--color-text-secondary); }
  .list-actions { flex-shrink: 0; }
}

.rom-list-item:nth-child(1) {
  --page-enter-delay: 0ms;
}

.rom-list-item:nth-child(2) {
  --page-enter-delay: 40ms;
}

.rom-list-item:nth-child(3) {
  --page-enter-delay: 80ms;
}

.rom-list-item:nth-child(4) {
  --page-enter-delay: 120ms;
}

.rom-list-item:nth-child(5) {
  --page-enter-delay: 160ms;
}
</style>
