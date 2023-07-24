<template>
  <div class="grid grid-cols-2">
    <div>{{ t('bracketView.grandFinals') }}</div>
    <div>{{ t('bracketView.bracketReset') }}</div>
    <MatchNode :match="bracket.grand_finals" />
    <MatchNode :match="bracket.grand_finals_reset" />
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Ref } from 'vue';
import MatchNode from '@/components/MatchNode.vue';
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

interface Match {
  id: string,
  players: string[],
  seeds: number[],
  score: number[],
  row_hint: number | null,
}

interface Bracket {
  winner_bracket: boolean,
  loser_bracket: boolean,
  grand_finals: Match | null,
  grand_finals_reset: Match | null,
}

const bracket: Ref<Bracket> = ref({
  winner_bracket: false,
  loser_bracket: false,
  grand_finals: null,
  grand_finals_reset: null,
})

onMounted(() => {
  bracket.value = JSON.parse(localStorage.getItem('bracket')!)
})

</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>