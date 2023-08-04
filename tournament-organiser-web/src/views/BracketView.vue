<template>
  <div
    class="pb-5 text-gray-400"
  >
    {{ t('bracketView.hint') }}
  </div>
  <div class="grid grid-cols-2">
    <div>{{ t('bracketView.grandFinals') }}</div>
    <div>{{ t('bracketView.bracketReset') }}</div>
    <MatchNode
      :match="bracket.grand_finals"
      class="max-w-[160px] "
    />
    <MatchNode
      :match="bracket.grand_finals_reset"
      class="max-w-[160px] "
    />
  </div>
  <div class="pt-6">
    <ShowBracket
      :bracket="bracket.winner_bracket"
      :lines="bracket.winner_bracket_lines"
    >
      {{ t('bracketView.winnerBracket') }}
    </ShowBracket>
  </div>
  <div class="pt-6">
    <ShowBracket 
      :bracket="bracket.loser_bracket"
      :lines="bracket.loser_bracket_lines"
    >
      {{ t('bracketView.loserBracket') }}
    </ShowBracket>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Ref } from 'vue';
import MatchNode from '@/components/MatchNode.vue';
import ShowBracket from '@/components/ShowBracket.vue';
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

const bracket: Ref<Bracket> = ref({
  winner_bracket: [],
  winner_bracket_lines: [],
  loser_bracket: [],
  loser_bracket_lines: [],
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