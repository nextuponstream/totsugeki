<template>
  <ReportResultModal v-model="show" :match-id="matchId" :players="players" />

  <div v-if="isGuest">{{ bracketName }}</div>
  <ExternalLink
    v-else
    :link-name="bracketStore.bracket?.bracket!.name"
  ></ExternalLink>
  <div class="pb-5 text-gray-400">
    {{ t('bracketView.hint') }}
  </div>
  <div v-if="unsavedBracketCanBeSaved">
    <SubmitBtn @click="saveAndRedirectToNewBracketPage"
      >{{ t('bracketView.saveBracket') }}
    </SubmitBtn>
  </div>
  <div v-else-if="userStore.id === null">
    {{ t('bracketView.unsavedWarning') }}
  </div>
  <div v-if="hasEnoughPlayersToDisplay">
    <ShowBracket
      :bracket="bracketStore.bracket?.winner_bracket"
      :lines="bracketStore.bracket?.winner_bracket_lines"
      :grand-finals="bracketStore.bracket?.grand_finals"
      :grand-finals-reset="bracketStore.bracket?.grand_finals_reset"
      test-id-prefix="winner"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.winnerBracket') }}
    </ShowBracket>
    <ShowBracket
      class="pt-6"
      :bracket="bracketStore.bracket?.loser_bracket"
      :lines="bracketStore.bracket?.loser_bracket_lines"
      test-id-prefix="loser"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.loserBracket') }}
    </ShowBracket>
  </div>
  <div v-else class="text-gray-500">
    {{ $t('bracketView.notEnoughPlayersToDisplay') }}
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted, computed, h } from 'vue'
import ShowBracket from '@/components/ShowBracket.vue'
import { useI18n } from 'vue-i18n'
import ReportResultModal from '@/components/ReportResultModal.vue'
import { useBracketStore } from '@/stores/bracket'
import { useRoute, useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'
import SubmitBtn from '@/components/ui/buttons/SubmitBtn.vue'
import { RouteNames } from '@/router'
import ExternalLink from '@/components/ui/ExternalLink.vue'

const bracketStore = useBracketStore()
const userStore = useUserStore()

const route = useRoute()
const router = useRouter()

const { t } = useI18n({})

const props = defineProps({
  isGuest: Boolean,
})

const unsavedBracketCanBeSaved = computed(() => {
  return userStore.id !== null && !bracketStore.isSaved
})

onMounted(async () => {
  // no bracket to fetch, guest view
  if (props.isGuest) {
    return
  }
  let id = route.params.bracketId
  if (typeof id === 'string') {
    bracketStore.setBracketId(id)
    await bracketStore.getDisplayableBracket()
  } else if (userStore.id === null && bracketStore.bracket) {
    // NOTE: when in dev, reloading a bracket page for the guest view might
    // throw the following error because pinia store is not reloaded before
    // component finishes loading even though it's fine?
    // Uncaught (in promise) Error: neither logged in view, nor guest view could
    // load properly
    //
    // guest view, nothing to do
  } else if (unsavedBracketCanBeSaved.value) {
    // guest just registered, they need to save that bracket
  } else {
    console.debug(typeof id)
    throw new Error(
      'neither logged in view, nor guest view could load properly'
    )
  }
})

const matchId = ref<string | null>(null)
const players = ref<{ name: string; id: string }[] | null>(null)
const show = ref(false)

function showResultModal(
  clickedMatchId: string,
  clickedPlayers: { name: string; id: string }[]
) {
  matchId.value = clickedMatchId
  players.value = clickedPlayers
  show.value = true
}

async function saveAndRedirectToNewBracketPage() {
  await bracketStore.saveBracket()
  if (bracketStore.bracket?.bracket?.id) {
    await router.push({
      name: RouteNames.bracket.show,
      params: { bracketId: bracketStore.bracket?.bracket.id },
    })
  } else {
    throw new Error('missing bracket id to redirect')
  }
}

const bracketName = computed(() => {
  return bracketStore.bracket?.bracket!.name
})

const hasEnoughPlayersToDisplay = computed(() => {
  if (bracketStore.bracket?.bracket?.participants?.length) {
    return bracketStore.bracket.bracket.participants.length >= 3
  }
  return false
})
</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>
