<template>
  <ReportResultModal 
    v-model="show"
    :match-id="matchId"
    :players="players"
  />

  <div class="pb-5 text-gray-400">
    {{ t('bracketView.hint') }}
  </div>
  <div>
    <ShowBracket
      :bracket="bracket.winner_bracket"
      :lines="bracket.winner_bracket_lines"
      :grand-finals="bracket.grand_finals"
      :grand-finals-reset="bracket.grand_finals_reset"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.winnerBracket') }}
    </ShowBracket>
  </div>
  <div class="pt-6">
    <ShowBracket 
      :bracket="bracket.loser_bracket"
      :lines="bracket.loser_bracket_lines"
      @show-result-modal="showResultModal"
    >
      {{ t('bracketView.loserBracket') }}
    </ShowBracket>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Ref } from 'vue';
import ShowBracket from '@/components/ShowBracket.vue';
import { useI18n } from 'vue-i18n';
import ReportResultModal from '@/components/ReportResultModal.vue';

const {t} = useI18n({})

const bracket: Ref<Bracket> = ref({
  winner_bracket: [],
  winner_bracket_lines: [],
  loser_bracket: [],
  loser_bracket_lines: [],
  grand_finals: undefined as Match | undefined,
  grand_finals_reset: undefined as Match | undefined,
})

onMounted(() => {
  bracket.value = JSON.parse(localStorage.getItem('bracket')!)
})

const matchId = ref<string | null>(null)
const players = ref<string[] | null>(null)
const show = ref(false)

function showResultModal(clickedMatchId: string, clickedPlayers: string[]) {
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