<template>
  <div class="text-2xl">
    Seeder
  </div>
  
  <draggable 
    v-if="players.length > 0"
    :list="players" 
    :disabled="!enabled"
    item-key="name"
    class="list-group"
    ghost-class="ghost"
    @start="dragging = true"
    @end="dragging = false"
  >
    <template #item="{element}">
      <div
        class="list-group-item"
        :class="{ 'not-draggable': !enabled }"
      >
        {{ element.name }}
      </div>
    </template>
  </draggable>
  <div v-else>
    <div>{{ t('playerSeeder.empty') }}</div>
  </div>
</template>
  
  <script setup lang="ts">
  import draggable from 'vuedraggable'
  import { type PropType } from 'vue'
  import { ref } from 'vue'
  import { useI18n } from 'vue-i18n';

  const {t} = useI18n({})

  interface Player {
    name: string,
  }
  
  const props = defineProps({
    players: { type: Array as PropType<Player[]>, required: false, default: () => {return []} }
  })
  
  const dragging = ref(false)
  const enabled = ref(true)
  
  </script>
  