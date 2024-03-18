<template>
  <div class="text-xl">
    {{ t('registration.bracketNameLabel') }}:
    {{ bracketStore.formCreate.bracket_name }}
  </div>
  <div class="sm:grid sm:grid-cols-2 sm:gap-5">
    <div class="pb-5">
      <player-registration @new-player="bracketStore.addPlayerInForm" />
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
            {{ t('playerRegistrationForm.minimum', { min: 3 }) }}
          </base-tooltip>
        </div>
      </div>
    </div>

    <div>
      <player-seeder />
    </div>
  </div>
</template>

<script setup lang="ts">
import PlayerSeeder from '@/components/PlayerSeeder.vue'
import PlayerRegistration from '@/components/PlayerRegistration.vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useBracketStore } from '@/stores/bracket'
import { useUserStore } from '@/stores/user'

const { t } = useI18n({})
const router = useRouter()

const bracketStore = useBracketStore()
const userStore = useUserStore()

const hasMinNumberOfPlayerToStartBracket = computed(() => {
  return bracketStore.formCreate.player_names.length < 3
})

async function createBracketFromPlayers() {
  let loggedIn: boolean = userStore.id !== null
  try {
    await bracketStore.createBracket(loggedIn)
    if (loggedIn) {
      await router.push({
        name: 'bracket',
        params: { bracketId: bracketStore.id },
      })
    } else {
      await router.push({ name: 'bracket-guest' })
    }
  } catch (e) {
    console.error(e)
  }
}
</script>
