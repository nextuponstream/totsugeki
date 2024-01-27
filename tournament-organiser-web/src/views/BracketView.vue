<template>
  <ReportResultModal
    v-model="show"
    :match-id="matchId"
    :players="players"
    @new-result="reportResult"
  />

  <div class="pb-5 text-gray-400">
    {{ t("bracketView.hint") }}
  </div>
  <div>
    <ShowBracket
      :bracket="bracket.winner_bracket"
      :lines="bracket.winner_bracket_lines"
      :grand-finals="bracket.grand_finals"
      :grand-finals-reset="bracket.grand_finals_reset"
      @show-result-modal="showResultModal"
    >
      {{ t("bracketView.winnerBracket") }}
    </ShowBracket>
  </div>
  <div class="pt-6">
    <ShowBracket
      :bracket="bracket.loser_bracket"
      :lines="bracket.loser_bracket_lines"
      @show-result-modal="showResultModal"
    >
      {{ t("bracketView.loserBracket") }}
    </ShowBracket>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { Ref } from "vue";
import ShowBracket from "@/components/ShowBracket.vue";
import { useI18n } from "vue-i18n";
import ReportResultModal from "@/components/ReportResultModal.vue";

const { t } = useI18n({});

const bracket: Ref<Bracket> = ref({
  winner_bracket: [],
  winner_bracket_lines: [],
  loser_bracket: [],
  loser_bracket_lines: [],
  grand_finals: undefined as Match | undefined,
  grand_finals_reset: undefined as Match | undefined,
  bracket: undefined,
});

onMounted(() => {
  bracket.value = JSON.parse(localStorage.getItem("bracket")!);
});

const matchId = ref<string | null>(null);
const players = ref<{ name: string; id: string }[] | null>(null);
const show = ref(false);

function showResultModal(
  clickedMatchId: string,
  clickedPlayers: { name: string; id: string }[]
) {
  matchId.value = clickedMatchId;
  players.value = clickedPlayers;
  show.value = true;
}

async function reportResult(
  players: { name: string; id: string }[],
  scoreP1: number,
  scoreP2: number
) {
  try {
    console.log(bracket.value.bracket);
    let response = await fetch(
      "https://totsugeki.fly.dev/report-result-for-bracket",
      {
        method: "POST",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          bracket: bracket.value.bracket,
          p1_id: players[0].id,
          p2_id: players[1].id,
          score_p1: scoreP1,
          score_p2: scoreP2,
        }),
      }
    );
    let newBracket = await response.json();
    localStorage.setItem("bracket", JSON.stringify(bracket));
    bracket.value = newBracket;
  } catch (e) {
    console.error(e);
  }
}
</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>
