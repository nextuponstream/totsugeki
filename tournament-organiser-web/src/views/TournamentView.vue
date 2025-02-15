<template>
  <ReportResultModal v-model="show" :match-id="matchId" :players="players" />
  <JoinBracketConfirmModal
    v-model="showJoin"
    @confirmed="joinBracket"
  ></JoinBracketConfirmModal>

  <div v-if="isGuest">{{ bracketName }}</div>
  <ExternalLink
    v-else
    :link-name="tournamentStore.bracket?.bracket!.name"
  ></ExternalLink>
  <div v-if="showJoinLink">
    <other-btn @click="showJoinModal">{{ $t('bracketView.join') }}</other-btn>
  </div>
  <div class="pb-5 text-gray-400">
    {{ t('bracketView.hint') }}
  </div>
  <div v-if="unsavedBracketCanBeSavedAction">
    <SubmitBtn @click="saveAndRedirectToNewBracketPage"
      >{{ t('bracketView.saveBracket') }}
    </SubmitBtn>
  </div>
  <div v-else-if="unsavedBracketCanBeSavedWarning">
    {{ t('bracketView.unsavedWarning') }}
  </div>
  <div v-if="hasEnoughPlayersToDisplay">
    <ShowBracket
      :bracket="tournamentStore.bracket?.winner_bracket"
      :lines="tournamentStore.bracket?.winner_bracket_lines"
      :grand-finals="tournamentStore.bracket?.grand_finals"
      :grand-finals-reset="tournamentStore.bracket?.grand_finals_reset"
      test-id-prefix="winner"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.winnerBracket') }}
    </ShowBracket>
    <ShowBracket
      class="pt-6"
      :bracket="tournamentStore.bracket?.loser_bracket"
      :lines="tournamentStore.bracket?.loser_bracket_lines"
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
import { useTournamentStore } from '@/stores/tournament'
import { useRoute, useRouter } from 'vue-router'
import { useUserStore } from '@/stores/user'
import SubmitBtn from '@/components/ui/buttons/SubmitBtn.vue'
import { RouteNames } from '@/router'
import ExternalLink from '@/components/ui/ExternalLink.vue'
import JoinBracketConfirmModal from '@/components/JoinBracketConfirmModal.vue'

const tournamentStore = useTournamentStore()
const userStore = useUserStore()

const route = useRoute()
const router = useRouter()

const { t } = useI18n({})

const props = defineProps({
  isGuest: Boolean,
})

const unsavedBracketCanBeSavedAction = computed(() => {
  return userStore.id !== null && !tournamentStore.isSaved
})

const unsavedBracketCanBeSavedWarning = computed(() => {
  return userStore.id === null && !tournamentStore.isSaved
})

onMounted(async () => {
  // no bracket to fetch, guest view
  if (props.isGuest) {
    return
  }
  let id = route.params.tournamentId
  if (typeof id === 'string') {
    tournamentStore.setTournamentId(id)
    await tournamentStore.getDisplayableTournament()
  } else if (userStore.id === null && tournamentStore.bracket) {
    // NOTE: when in dev, reloading a bracket page for the guest view might
    // throw the following error because pinia store is not reloaded before
    // component finishes loading even though it's fine?
    // Uncaught (in promise) Error: neither logged in view, nor guest view could
    // load properly
    //
    // guest view, nothing to do
  } else if (unsavedBracketCanBeSavedWarning.value) {
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
  await tournamentStore.saveTournament()
  if (tournamentStore.bracket?.tournament?.id) {
    await router.push({
      name: RouteNames.tournaments.show,
      params: { tournamentId: tournamentStore.bracket.tournament.id },
    })
  } else {
    throw new Error('missing tournament id to redirect')
  }
}

const bracketName = computed(() => {
  return tournamentStore.bracket?.tournament?.name
})

const hasEnoughPlayersToDisplay = computed(() => {
  if (tournamentStore.participants?.length) {
    return tournamentStore.participants?.length >= 3
  }
  return false
})

const showJoinLink = computed(() => {
  return (
    !tournamentStore.bracket?.is_participant &&
    !tournamentStore.bracket?.is_tournament_organiser
  )
})

const showJoin = ref(false)

function showJoinModal() {
  showJoin.value = true
}

async function joinBracket() {
  showJoin.value = false
  await tournamentStore.join()
}
</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>
