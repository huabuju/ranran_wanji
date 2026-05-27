<template>
  <div class="content-area surface-card-strong">
    <el-empty v-if="commands.length === 0" description="暂无自定义指令，请点击右上角新增">
      <el-button type="primary" @click="$emit('add-command')">新增指令</el-button>
    </el-empty>

    <VueDraggable
      v-else
      v-model="localCommands"
      class="cards-grid"
      animation="300"
      ghost-class="drag-ghost"
      :force-fallback="true"
      :fallback-tolerance="3"
      :disabled="true"
      @end="$emit('sort-end')"
    >
      <CommandCard
        v-for="item in localCommands"
        :key="item.id"
        :item="item"
        :is-batch-mode="isBatchMode"
        :is-selected="isSelected(item.id)"
        @click="$emit('card-click', item)"
        @edit="$emit('edit-command', item)"
        @delete="$emit('delete-command', item)"
        @execute="$emit('execute-command', item)"
      />
    </VueDraggable>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';
import { VueDraggable } from 'vue-draggable-plus';
import CommandCard from './CommandCard.vue';

const props = defineProps({
  commands: {
    type: Array,
    default: () => [],
  },
  isBatchMode: {
    type: Boolean,
    default: false,
  },
  isSelected: {
    type: Function,
    required: true,
  },
});

const emit = defineEmits(['add-command', 'sort-end', 'card-click', 'edit-command', 'delete-command', 'execute-command']);

const localCommands = ref([]);

watch(() => props.commands, (value) => {
  localCommands.value = value;
}, { immediate: true });

watch(localCommands, (value) => {
  if (value !== props.commands) {
    emit('sort-end');
  }
});
</script>

<style lang="scss" scoped>
@use '@/assets/styles/_page-card.scss' as pageCard;

.content-area {
  flex: 1;
  overflow-y: auto;
  padding: 16px 16px 24px;
  display: flex;
  flex-direction: column;
  @include pageCard.overview-main-card-hoverable(var(--bg-glass), null);
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  align-content: start;
  min-height: 200px;
}

.cards-grid > *:nth-child(1) {
  --page-enter-delay: 0ms;
}

.cards-grid > *:nth-child(2) {
  --page-enter-delay: 40ms;
}

.cards-grid > *:nth-child(3) {
  --page-enter-delay: 80ms;
}

.cards-grid > *:nth-child(4) {
  --page-enter-delay: 120ms;
}

.cards-grid > *:nth-child(5) {
  --page-enter-delay: 160ms;
}

.cards-grid > *:nth-child(6) {
  --page-enter-delay: 200ms;
}

.drag-ghost {
  opacity: 0.4;
  border: 2px dashed var(--color-primary) !important;
  background: rgba(var(--color-primary-rgb), 0.05) !important;
  transform: scale(0.95);
}
</style>
