<template>
  <div class="text-xl">
    {{ t('registration.bracketNameLabel') }}: {{ bracketName }}
  </div>
  <div class="grid grid-cols-2">
    <div>
      <player-registration @new-player="addPlayer" />
      <div class="group mt-5 items-center flex">
        <submit-btn
          :disabled="hasMinNumberOfPlayerToStartBracket"
          @click="createBracketFromPlayers"
        >
          {{ t('registration.startBracket') }}
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
  import { useRouter } from 'vue-router'
import config from '@/config';
  
  const {t} = useI18n({})
  const router = useRouter()

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
  
  async function createBracketFromPlayers(){
    // FIXME remove 'no-cors' after development is over
    try {
      let response = await fetch('http://localhost:3000/bracket-from-players', config.axumHeaders)
      console.log(JSON.stringify(response))
      let bracket = await response.json()
      console.log(JSON.stringify(bracket)) // FIXME still empty with npm run dev
      localStorage.setItem('bracket', JSON.stringify(bracket))
    } catch (e) {
      console.error(e)
    }

    // TODO fix cors issue with npm run dev when not using mode:no-cors
    // FIXME can't visit /registration directly

    router.push({
     name: 'bracket',
    })
  }
  </script>
  