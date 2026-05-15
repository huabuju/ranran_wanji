<template>
  <GlassLoading
    :show="isChecking"
    title="正在检查更新..."
    description="请稍候，我们正在连接远程服务器"
  >
    <template #icon>
      <el-icon class="pulse-icon"><Search /></el-icon>
    </template>
  </GlassLoading>
</template>

<script setup>
import { useUpdateStore } from '@/utils/updateStore';
import { Search } from '@element-plus/icons-vue';
import GlassLoading from './GlassLoading.vue';

const { isChecking } = useUpdateStore();
</script>

<style lang="scss" scoped>
.pulse-icon {
  font-size: 26px;
  color: var(--brand-primary-strong);
  animation: icon-pulse 1.5s ease-in-out infinite;
}

@keyframes icon-pulse {
  0%, 100% { transform: scale(1); opacity: 0.8; }
  50% { transform: scale(1.15); opacity: 1; }
}
</style>
