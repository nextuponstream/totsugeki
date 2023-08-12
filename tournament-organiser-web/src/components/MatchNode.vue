<template>
  <div
    class="py-2 flex flex-col"   
    :class="rowClass" 
  >
    <div
      class="flex flex-col my-auto"
      :class="matchClass"
    >
      <div 
        class="flex"
        :class="verticalSeparator"
      >
        <div>{{ !isPaddingMatch ? match?.seeds[0] : '&#8205;' }}</div>
        <div class="grow pl-1">
          {{ match?.players[0].name }}
        </div>
        <MatchScore
          :is-padding-match="isPaddingMatch"
          :scores="match?.score"
          :index="0"
          :other-index="1"
        />
      </div>
      <div 
        class="flex"
        :class="verticalSeparator"
      >
        <div>{{ !isPaddingMatch ? match?.seeds[1] : '&#8205;' }}</div>
        <div class="grow pl-1">
          {{ match?.players[1].name }}
        </div>
        <MatchScore
          :is-padding-match="isPaddingMatch"
          :scores="match?.score"
          :index="1"
          :other-index="0"
        />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import MatchScore from './match/MatchScore.vue'

// TODO hover only for matches for which you can submit scores

interface Match {
  id: string,
  players: {name: string, id: string}[],
  seeds: number[],
  score: number[],
  row_hint: number | null,
}

const props = defineProps<{
    match: Match | undefined,
}>()

const isPaddingMatch = computed(() => {
  return props.match?.row_hint == null && props.match?.seeds[0] == 0
})

const matchClass = computed(() => {
   if (isPaddingMatch.value) {
    return null
  } else {
    return 'flex flex-col divide-y border-1 border-box border hover:bg-gray-300'
  }
})

const rowClass = computed(() => {
  return `${props.match?.row_hint != null ? `row-start-${props.match.row_hint + 1}` : ''}`
})

const verticalSeparator = computed(() => {
  if (isPaddingMatch.value) {
    return null
  } else {
    return 'divide-x'
  }
})
</script>