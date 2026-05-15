<template>
  <component :is="'div'" class="file-icon-wrap" :class="[sizeClass, { 'is-grid': isGrid }]">
    <!-- 文件夹 -->
    <svg v-if="isDir" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <path d="M4 10C4 7.79086 5.79086 6 8 6H20L24 12H40C42.2091 12 44 13.7909 44 16V38C44 40.2091 42.2091 42 40 42H8C5.79086 42 4 40.2091 4 38V10Z" fill="#FFA940"/>
      <path d="M4 18H44V38C44 40.2091 42.2091 42 40 42H8C5.79086 42 4 40.2091 4 38V18Z" fill="#FFD666"/>
    </svg>

    <!-- APK -->
    <svg v-else-if="ext === 'apk'" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="4" y="2" width="40" height="44" rx="4" fill="#E8F5E9"/>
      <rect x="4" y="2" width="40" height="44" rx="4" stroke="#4CAF50" stroke-width="1.5"/>
      <path d="M15 30C15 24.4772 19.4772 20 25 20C30.5228 20 35 24.4772 35 30" stroke="#4CAF50" stroke-width="2.5" stroke-linecap="round"/>
      <circle cx="18" cy="22" r="2" fill="#4CAF50"/>
      <circle cx="32" cy="22" r="2" fill="#4CAF50"/>
      <path d="M20 34H30M24 30V38" stroke="#4CAF50" stroke-width="2" stroke-linecap="round"/>
    </svg>

    <!-- 图片 -->
    <svg v-else-if="isImage" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="4" y="6" width="40" height="36" rx="4" fill="#E3F2FD"/>
      <rect x="4" y="6" width="40" height="36" rx="4" stroke="#2196F3" stroke-width="1.5"/>
      <circle cx="16" cy="18" r="4" fill="#FFA726"/>
      <path d="M4 32L16 20L25 29L32 22L44 34V38C44 40.2091 42.2091 42 40 42H8C5.79086 42 4 40.2091 4 38V32Z" fill="#42A5F5"/>
    </svg>

    <!-- 视频 -->
    <svg v-else-if="isVideo" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="4" y="6" width="40" height="36" rx="4" fill="#E8EAF6"/>
      <rect x="4" y="6" width="40" height="36" rx="4" stroke="#3F51B5" stroke-width="1.5"/>
      <path d="M32 20L40 16V32L32 28V20Z" fill="#3F51B5"/>
      <rect x="8" y="16" width="22" height="16" rx="2" fill="#3F51B5"/>
    </svg>

    <!-- 音频 -->
    <svg v-else-if="isAudio" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="4" y="2" width="40" height="44" rx="4" fill="#F3E5F5"/>
      <rect x="4" y="2" width="40" height="44" rx="4" stroke="#9C27B0" stroke-width="1.5"/>
      <circle cx="24" cy="26" r="8" fill="#9C27B0"/>
      <circle cx="24" cy="26" r="3" fill="white"/>
      <path d="M24 18V12" stroke="#9C27B0" stroke-width="2" stroke-linecap="round"/>
      <path d="M24 12L34 16" stroke="#9C27B0" stroke-width="2" stroke-linecap="round"/>
    </svg>

    <!-- PDF -->
    <svg v-else-if="ext === 'pdf'" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="6" y="2" width="36" height="44" rx="3" fill="#FFEBEE"/>
      <rect x="6" y="2" width="36" height="44" rx="3" stroke="#F44336" stroke-width="1.5"/>
      <path d="M6 30H42V43C42 44.6569 40.6569 46 39 46H9C7.34315 46 6 44.6569 6 43V30Z" fill="#F44336"/>
      <text x="10" y="43" font-size="9" fill="white" font-weight="bold" font-family="Arial">PDF</text>
      <rect x="14" y="10" width="20" height="2" rx="1" fill="#F44336"/>
      <rect x="14" y="15" width="20" height="2" rx="1" fill="#F44336"/>
      <rect x="14" y="20" width="14" height="2" rx="1" fill="#F44336"/>
    </svg>

    <!-- ZIP/压缩包 -->
    <svg v-else-if="isArchive" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="6" y="2" width="36" height="44" rx="3" fill="#FFF3E0"/>
      <rect x="6" y="2" width="36" height="44" rx="3" stroke="#FF9800" stroke-width="1.5"/>
      <rect x="20" y="2" width="8" height="44" fill="#FFE0B2"/>
      <rect x="20" y="8" width="8" height="5" rx="1" fill="#FF9800"/>
      <rect x="20" y="18" width="8" height="5" rx="1" fill="#FF9800"/>
      <rect x="20" y="28" width="8" height="5" rx="1" fill="#FF9800"/>
      <circle cx="24" cy="38" r="3" fill="#FF9800"/>
    </svg>

    <!-- JSON -->
    <svg v-else-if="ext === 'json'" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="6" y="2" width="36" height="44" rx="3" fill="#E3F2FD"/>
      <rect x="6" y="2" width="36" height="44" rx="3" stroke="#1565C0" stroke-width="1.5"/>
      <text x="9" y="22" font-size="11" fill="#1565C0" font-family="monospace" font-weight="bold">{  }</text>
      <rect x="10" y="26" width="28" height="2" rx="1" fill="#1565C0" opacity="0.4"/>
      <rect x="10" y="31" width="20" height="2" rx="1" fill="#1565C0" opacity="0.4"/>
      <rect x="10" y="36" width="24" height="2" rx="1" fill="#1565C0" opacity="0.4"/>
    </svg>

    <!-- JS/TS/Vue/CSS/HTML 等代码文件 -->
    <svg v-else-if="isCode" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="6" y="2" width="36" height="44" rx="3" :fill="codeIconBg"/>
      <rect x="6" y="2" width="36" height="44" rx="3" :stroke="codeIconColor" stroke-width="1.5"/>
      <path d="M18 18L10 24L18 30" :stroke="codeIconColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M30 18L38 24L30 30" :stroke="codeIconColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M26 14L22 34" :stroke="codeIconColor" stroke-width="2.5" stroke-linecap="round"/>
    </svg>

    <!-- Shell 脚本 -->
    <svg v-else-if="ext === 'sh'" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <rect x="6" y="2" width="36" height="44" rx="3" fill="#E8F5E9"/>
      <rect x="6" y="2" width="36" height="44" rx="3" stroke="#388E3C" stroke-width="1.5"/>
      <rect x="6" y="2" width="36" height="14" rx="3" fill="#388E3C"/>
      <circle cx="14" cy="9" r="2" fill="#EF5350"/>
      <circle cx="21" cy="9" r="2" fill="#FFC107"/>
      <circle cx="28" cy="9" r="2" fill="#66BB6A"/>
      <path d="M12 24L18 30" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
      <path d="M12 30L18 24" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
      <path d="M22 30H32" stroke="#388E3C" stroke-width="2" stroke-linecap="round"/>
    </svg>

    <!-- 通用文本文件 -->
    <svg v-else viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon-svg">
      <path d="M8 4H32L40 12V44C40 45.1046 39.1046 46 38 46H10C8.89543 46 8 45.1046 8 44V4Z" fill="#FAFAFA"/>
      <path d="M8 4H32L40 12V44C40 45.1046 39.1046 46 38 46H10C8.89543 46 8 45.1046 8 44V4Z" stroke="#B0BEC5" stroke-width="1.5"/>
      <path d="M32 4L32 12H40" stroke="#B0BEC5" stroke-width="1.5"/>
      <rect x="14" y="20" width="20" height="2" rx="1" fill="#CFD8DC"/>
      <rect x="14" y="26" width="20" height="2" rx="1" fill="#CFD8DC"/>
      <rect x="14" y="32" width="14" height="2" rx="1" fill="#CFD8DC"/>
    </svg>
  </component>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  name: { type: String, default: '' },
  isDir: { type: Boolean, default: false },
  size: { type: Number, default: 32 }, // px
  isGrid: { type: Boolean, default: false },
});

// 提取扩展名（小写）
const ext = computed(() => {
  const parts = props.name.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
});

const isImage = computed(() => ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg', 'heic', 'heif'].includes(ext.value));
const isVideo = computed(() => ['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv', 'webm', 'm4v', 'ts'].includes(ext.value));
const isAudio = computed(() => ['mp3', 'flac', 'wav', 'aac', 'ogg', 'm4a', 'wma', 'opus'].includes(ext.value));
const isArchive = computed(() => ['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'tgz'].includes(ext.value));
const isCode = computed(() => ['js', 'ts', 'vue', 'jsx', 'tsx', 'css', 'scss', 'html', 'htm', 'py', 'java', 'kt', 'rs', 'go', 'c', 'cpp', 'h', 'xml', 'dart'].includes(ext.value));

const sizeClass = computed(() => {
  if (props.size <= 20) return 'icon-xs';
  if (props.size <= 32) return 'icon-sm';
  if (props.size <= 48) return 'icon-md';
  return 'icon-lg';
});

// 代码文件图标颜色
const codeColorMap = {
  js: { bg: '#FFFDE7', color: '#F9A825' },
  ts: { bg: '#E3F2FD', color: '#1976D2' },
  vue: { bg: '#E8F5E9', color: '#42B883' },
  jsx: { bg: '#E3F2FD', color: '#61DAFB' },
  tsx: { bg: '#E3F2FD', color: '#61DAFB' },
  css: { bg: '#E3F2FD', color: '#2196F3' },
  scss: { bg: '#FCE4EC', color: '#E91E8C' },
  html: { bg: '#FBE9E7', color: '#E64A19' },
  htm: { bg: '#FBE9E7', color: '#E64A19' },
  py: { bg: '#E3F2FD', color: '#1565C0' },
  java: { bg: '#FFF3E0', color: '#E65100' },
  kt: { bg: '#F3E5F5', color: '#7B1FA2' },
  rs: { bg: '#FBE9E7', color: '#BF360C' },
  xml: { bg: '#E8F5E9', color: '#2E7D32' },
};
const codeIconBg = computed(() => codeColorMap[ext.value]?.bg || '#F5F5F5');
const codeIconColor = computed(() => codeColorMap[ext.value]?.color || '#9E9E9E');
</script>

<style scoped>
.file-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.icon-xs .icon-svg { width: 18px; height: 18px; }
.icon-sm .icon-svg { width: 28px; height: 28px; }
.icon-md .icon-svg { width: 40px; height: 40px; }
.icon-lg .icon-svg { width: 64px; height: 64px; }

/* 网格模式下字体标注增大并居中 */
.is-grid .icon-svg {
  width: 64px;
  height: 64px;
  filter: drop-shadow(0 2px 6px rgba(0,0,0,0.1));
}
</style>
