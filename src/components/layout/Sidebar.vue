<template>
  <aside class="sidebar">
    <div class="sidebar-top">
      <div class="sidebar-logo">
        <div class="logo-container">
          <SmartIcon 
            name="logo" 
            color="white" 
            :size="32"
            :show-background="false"
            :show-decoration="true"
          />
        </div>
        <span class="sidebar-title"> 然然玩机工具箱</span>
      </div>
    </div>

    <el-menu
      :default-active="route.path"
      router
      class="sidebar-nav"
      :collapse="false"
    >
      <el-menu-item 
        v-for="item in menuItems" 
        :key="item.path" 
        :index="item.path"
      >
        <SmartIcon 
          :name="item.icon" 
          :color="item.color" 
          :size="18"
          class="nav-icon"
        />
        <template #title>
          <span class="nav-label">{{ item.label }}</span>
        </template>
      </el-menu-item>
    </el-menu>
  </aside>
</template>

<script setup>
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import SmartIcon from '@/components/common/SmartIcon.vue';

const router = useRouter();
const route = useRoute();

// 从路由配置中动态获取菜单项
const menuItems = computed(() => {
  // 获取所有一级路由
  return router.options.routes.map(item => ({
    path: item.path,
    label: item.meta?.title || item.name,
    icon: item.meta?.icon,
    color: item.meta?.color
  }));
});

</script>

<style lang="scss" scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100%;
  background: var(--bg-sidebar);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  z-index: 20;

  /* Top Logo Area */
  .sidebar-top {
    padding: 20px 12px 16px;
    border-bottom: 1px solid var(--color-divider);

    .sidebar-logo {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 10px;

      .logo-container {
        width: 64px;
        height: 64px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--primary-gradient);
        border-radius: 18px; /* Slightly more rounded */
        box-shadow: var(--primary-shadow-card);
        margin-bottom: 8px;
        transition: transform 0.3s ease;

        &:hover {
          transform: translateY(-2px) scale(1.02);
        }
      }

      .sidebar-title {
        font-size: 12px;
        font-weight: 600;
        color: var(--color-text-primary);
        text-align: center;
        line-height: 1.4;
      }
    }
  }

  /* Nav Items (Element Plus overrides) */
  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    border-right: none !important;
    background: transparent;
    padding: 8px 0;

    :deep(.el-menu-item) {
      display: flex !important;
      align-items: center;
      gap: 10px;
      height: 42px !important;
      line-height: 42px !important;
      padding: 0 16px !important;
      margin: 4px 12px !important;
      border-radius: var(--radius-md);
      color: var(--color-text-secondary);
      font-size: 13px;
      transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
      position: relative;
      overflow: hidden;

      &:hover {
        background: var(--sidebar-item-hover-bg) !important;
        color: var(--color-text-primary) !important;
      }

      &.is-active {
        background: linear-gradient(90deg, var(--sidebar-item-active-bg), transparent) !important;
        color: var(--sidebar-item-active-color) !important;
        font-weight: 600;

        &::before {
          content: '';
          position: absolute;
          left: 0;
          top: 50%;
          transform: translateY(-50%);
          height: 50%;
          width: 4px;
          background: var(--color-primary);
          border-radius: 4px;
        }
      }

      .nav-icon {
        flex-shrink: 0;
      }

      &:hover .nav-icon {
        transform: scale(1.1);
      }

      .nav-label {
        line-height: 1.2;
      }
    }
  }
}
</style>
