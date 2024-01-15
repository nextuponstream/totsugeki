<template>
  <div class="text-xl">
    {{ t('registration.bracketNameLabel') }}: {{ bracketName }}
  </div>
  <div class="sm:grid sm:grid-cols-2 sm:gap-5">
    <div class="pb-5">
      <player-registration @new-player="addPlayer" />
      <div class="group mt-5 grid grid-cols-1 place-items-center">
        <div>
          <submit-btn
            :disabled="hasMinNumberOfPlayerToStartBracket"
            @click="createBracketFromPlayers"
          >
            {{ t('registration.startBracket') }}
          </submit-btn>
          <base-tooltip
            v-if="hasMinNumberOfPlayerToStartBracket"
            class="ml-3"
            style="position: absolute"
          >
            3 players minimum
          </base-tooltip>
        </div>
      </div>
    </div>

    <div>
      <player-seeder
        :players="playerList"
        @remove-player="removePlayer"
      />
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
  
  const playerList : Ref<{name: string, index: number}[]>= ref([])
  
  const dragging = ref(false)
  const enabled = ref(true)
  const counter = ref(0)
  
  function addPlayer(name: string): void {
    // index is used as vue key. Because it must be unique, then we tie it to some independent counter
    // rather than playerList size (which varies when removing player)
    counter.value = counter.value + 1
    playerList.value.push({name: name, index: counter.value})
  }

  function removePlayer(index: number): void {
      let player = playerList.value.findIndex(p => p.index = index)
      if (player > -1) {
        playerList.value.splice(player, 1)
      }
  }
  
  const hasMinNumberOfPlayerToStartBracket = computed(() => {
    return playerList.value.length < 3
  })
  
  async function createBracketFromPlayers(){
    try {
      // TODO configurable variable
      let response = await fetch('https://totsugeki.fly.dev/bracket-from-players', {
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
  