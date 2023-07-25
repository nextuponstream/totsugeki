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
    try {
      console.log(JSON.stringify(playerList.value.map(p => p.name)))
      let response = await fetch('http://localhost:3000/bracket-from-players', {
        method: 'POST',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({names: playerList.value.map(p => p.name)}),
        // can't send json without cors... https://stackoverflow.com/a/45655314
        // documentation: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch#supplying_request_options
      })
      let bracket = await response.json()
      localStorage.setItem('bracket', JSON.stringify(bracket))
    } catch (e) {
      console.error(e)
    }

    router.push({
     name: 'bracket',
    })
  }
  </script>
  