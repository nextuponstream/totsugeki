<template>
  <div class="text-2xl">
    {{ t('playerSeeder.title') }}
  </div>
  <div class="text-gray-400">
    {{ t('playerSeeder.hint') }}
  </div>
  
  <draggable 
    v-if="players.length > 0"
    :list="players" 
    :disabled="!enabled"
    item-key="index"
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
        <div class="flex py-1 justify-between w-64">
          <div>{{ element.name }}</div>
          <i
            class="pi pi-times text-gray-400 hover:text-gray-800 py-1"
            @click="removePlayer(element.index)"
          />
        </div>
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
    index: number,
  }
  
  const props = defineProps({
    players: { type: Array as PropType<Player[]>, required: false, default: () => {return []} }
  })

  const emit = defineEmits(['removePlayer'])
  
  const dragging = ref(false)
  const enabled = ref(true)

  function removePlayer(index: number) {
    emit('removePlayer', index)
  }
  
  </script>
  