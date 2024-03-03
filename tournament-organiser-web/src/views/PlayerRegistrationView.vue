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
            data-test-id="start-bracket"
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
            {{ $t('playerRegistrationForm.minimum', { min: 3 }) }}
          </base-tooltip>
        </div>
      </div>
    </div>

    <div>
      <player-seeder :players="playerList" @remove-player="removePlayer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import PlayerSeeder from '@/components/PlayerSeeder.vue'
import PlayerRegistration from '@/components/PlayerRegistration.vue'
import { computed, ref, onMounted } from 'vue'
import type { Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useBracketStore } from '@/stores/bracket'
import { useUserStore } from '@/stores/user'

const { t } = useI18n({})
const router = useRouter()

const bracketName = ref('')
const bracketStore = useBracketStore()
const userStore = useUserStore()

onMounted(() => {
  bracketName.value = localStorage.getItem('bracketName') ?? ''
})

const playerList: Ref<{ name: string; index: number }[]> = ref([])
const dragging = ref(false)
const enabled = ref(true)
const counter = ref(0)

function addPlayer(name: string): void {
  // index is used as vue key. Because it must be unique, then we tie it to some independent counter
  // rather than playerList size (which varies when removing player)
  counter.value = counter.value + 1
  playerList.value.push({ name: name, index: counter.value })
}

function removePlayer(index: number): void {
  let player = playerList.value.findIndex((p) => (p.index = index))
  if (player > -1) {
    playerList.value.splice(player, 1)
  }
}

const hasMinNumberOfPlayerToStartBracket = computed(() => {
  return playerList.value.length < 3
})

async function createBracketFromPlayers() {
  let loggedIn: boolean = userStore.id !== null
  try {
    await bracketStore.createBracket(playerList.value, loggedIn)
    if (loggedIn) {
      router.push({ name: 'bracket', params: { bracketId: bracketStore.id } })
    } else {
      router.push({ name: 'bracket-guest' })
    }
  } catch (e) {
    console.error(e)
  }
}
</script>
