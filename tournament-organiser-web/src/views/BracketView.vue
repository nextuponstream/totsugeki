<template>
  <!--   <div class="col-span-1 flex flex-col my-auto divide-x divide-y border-1 border-box border hover:bg-gray-300 rounded-md">
    <div class="match flex flex-row divide-x">
      <div>{{ bracket.grand_finals?.seeds[0] }}</div>
      <div class="grow pl-1">{{ bracket.grand_finals?.players[0] }}</div>
      <div>{{ bracket.grand_finals?.score[0] }}</div>
    </div>
    <div class="match flex flex-row divide-x">
      <div>{{ bracket.grand_finals?.seeds[1] }}</div>
      <div class="grow pl-1">{{ bracket.grand_finals?.players[1] }}</div>
      <div>{{ bracket.grand_finals?.score[1] }}</div>
    </div>
  </div> -->
  <MatchNode :match="bracket.grand_finals" />
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Ref } from 'vue';
import MatchNode from '@/components/MatchNode.vue';

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