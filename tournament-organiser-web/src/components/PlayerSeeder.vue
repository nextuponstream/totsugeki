<template>
  <div class="flex gap-10">
    <div class="text-2xl">
      {{ t('playerSeeder.title') }}
    </div>
    <DangerBtn
      :disabled="noPlayers"
      @click="bracketStore.removeAllPlayersInForm"
      >{{ $t('playerSeeder.removeAllPlayers') }}
    </DangerBtn>
  </div>
  <div class="text-gray-400">
    {{ t('playerSeeder.hint') }}
  </div>

  <draggable
    v-if="bracketStore.formCreate.player_names.length > 0"
    :list="bracketStore.formCreate.player_names"
    :disabled="!enabled"
    item-key="index"
    class="list-group"
    ghost-class="ghost"
    @start="dragging = true"
    @end="dragging = false"
  >
    <template #item="{ element }">
      <div class="list-group-item" :class="{ 'not-draggable': !enabled }">
        <div class="flex py-1 justify-between w-64">
          <div>{{ element.name }}</div>
          <i
            class="pi pi-times text-gray-400 hover:text-gray-800 py-1"
            @click="bracketStore.removePlayerInForm(element.index)"
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
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useBracketStore } from '@/stores/bracket'
import DangerBtn from './ui/buttons/DangerBtn.vue'

const { t } = useI18n({})
const bracketStore = useBracketStore()
const dragging = ref(false)
const enabled = ref(true)
const noPlayers = computed(() => {
  return bracketStore.formCreate.player_names.length === 0
})
</script>
