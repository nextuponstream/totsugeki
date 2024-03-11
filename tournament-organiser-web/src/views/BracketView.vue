<template>
  <ReportResultModal v-model="show" :match-id="matchId" :players="players" />

  <div>{{ JSON.stringify(bracketStore.bracket?.bracket!.name) }}</div>
  <div class="pb-5 text-gray-400">
    {{ t('bracketView.hint') }}
  </div>
  <div v-if="unsavedBracketCanBeSaved">
    <SubmitBtn>Save bracket!!!</SubmitBtn>
  </div>
  <div v-else-if="userStore.id === null">
    {{ t('bracketView.unsavedWarning') }}
  </div>
  <div>
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
  </div>
  <div class="pt-6">
    <ShowBracket
      :bracket="bracketStore.bracket?.loser_bracket"
      :lines="bracketStore.bracket?.loser_bracket_lines"
      test-id-prefix="loser"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.loserBracket') }}
    </ShowBracket>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import ShowBracket from '@/components/ShowBracket.vue'
import { useI18n } from 'vue-i18n'
import ReportResultModal from '@/components/ReportResultModal.vue'
import { useBracketStore } from '@/stores/bracket'
import { useRoute } from 'vue-router'
import { useUserStore } from '@/stores/user'
import SubmitBtn from '@/components/ui/SubmitBtn.vue'
const bracketStore = useBracketStore()
const userStore = useUserStore()

const route = useRoute()

const { t } = useI18n({})

const unsavedBracketCanBeSaved = computed(() => {
  return userStore.id && !bracketStore.isSaved
})

onMounted(async () => {
  let id = route.params.bracketId
  if (typeof id === 'string') {
    bracketStore.setBracketId(id)
    await bracketStore.getDisplayableBracket()
  } else if (userStore.id === null && bracketStore.bracket) {
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
</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>
