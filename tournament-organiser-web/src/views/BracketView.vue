<template>
  <ReportResultModal
    v-model="show"
    :match-id="matchId"
    :players="players"
    @new-result="reportResult"
  />

  <div class="pb-5 text-gray-400">
    {{ t('bracketView.hint') }}
  </div>
  <div>
    <ShowBracket
      :bracket="bracketStore.bracket?.winner_bracket"
      :lines="bracketStore.bracket?.winner_bracket_lines"
      :grand-finals="bracketStore.bracket?.grand_finals"
      :grand-finals-reset="bracketStore.bracket?.grand_finals_reset"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.winnerBracket') }}
    </ShowBracket>
  </div>
  <div class="pt-6">
    <ShowBracket
      :bracket="bracketStore.bracket?.loser_bracket"
      :lines="bracketStore.bracket?.loser_bracket_lines"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.loserBracket') }}
    </ShowBracket>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Ref } from 'vue'
import ShowBracket from '@/components/ShowBracket.vue'
import { useI18n } from 'vue-i18n'
import ReportResultModal from '@/components/ReportResultModal.vue'
import { useBracketStore } from '@/stores/bracket'

const bracketStore = useBracketStore()

const { t } = useI18n({})

onMounted(async () => {
  await bracketStore.getDisplayableBracket()
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

async function reportResult(
  players: { name: string; id: string }[],
  scoreP1: number,
  scoreP2: number
) {
  try {
    // console.log(bracketStore.bracket.bracket)
    // let response = await fetch(
    //   `${import.meta.env.VITE_API_URL}/api/report-result-for-bracket`,
    //   {
    //     method: 'POST',
    //     headers: {
    //       Accept: 'application/json',
    //       'Content-Type': 'application/json',
    //     },
    //     body: JSON.stringify({
    //       bracket: bracket.value.bracket,
    //       p1_id: players[0].id,
    //       p2_id: players[1].id,
    //       score_p1: scoreP1,
    //       score_p2: scoreP2,
    //     }),
    //   }
    // )
    // if (response.ok) {
    //   let newBracket = await response.json()
    //   localStorage.setItem('bracket', JSON.stringify(bracket))
    //   bracket.value = newBracket
    // } else {
    //   throw new Error('non-200 response for /api/report-result-for-bracket')
    // }
  } catch (e) {
    console.error(e)
  }
}
</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>
