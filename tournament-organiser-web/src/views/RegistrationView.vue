<template>
  <div class="text-xl">
    {{ t('home.bracketNameLabel') }}: {{ bracketName }}
  </div>
  <div class="grid grid-cols-2">
    <div>
      <player-registration @new-player="addPlayer" />
      <div class="group mt-5 items-center flex">
        <submit-btn
          :disabled="hasMinNumberOfPlayerToStartBracket"
          @click="openConfirmModal"
        >
          {{ t('home.startBracket') }}
        </submit-btn>
        <base-tooltip
          v-if="hasMinNumberOfPlayerToStartBracket"
          class="ml-3"
        >
          3 players minimum
        </base-tooltip>
      </div>
    </div>

    <div>
      <player-seeder :players="playerList" />
    </div>
  </div>
</template>
  
  <script setup lang="ts">
  import PlayerSeeder from '@/components/PlayerSeeder.vue'
  import PlayerRegistration from '@/components/PlayerRegistration.vue'
  import { computed, ref, onMounted } from 'vue'
  import type { Ref } from 'vue'
  import { useI18n } from 'vue-i18n';
  
  const {t} = useI18n({})

  const bracketName = ref('')

  onMounted(() => {
    bracketName.value = localStorage.getItem('bracketName') ?? ''
  }) 
  
  const playerList : Ref<{name: string}[]>= ref([])
  
  const dragging = ref(false)
  const enabled = ref(true)
  
  function addPlayer(name: string): void {
    playerList.value.push({name: name})
  }
  
  const hasMinNumberOfPlayerToStartBracket = computed(() => {
    return playerList.value.length < 3
  })
  
  function openConfirmModal(){
    // TODO
  }
  </script>
  