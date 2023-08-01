<template>
  <div
    class="max-w-[140px] flex flex-col"
    :class="matchClass"
  >
    <div 
      class="flex"
      :class="verticalSeparator"
    >
      <div>{{ !isPaddingMatch ? match?.seeds[0] : '&#8205;' }}</div>
      <div class="grow pl-1">
        {{ match?.players[0] }}
      </div>
      <div>{{ !isPaddingMatch ? match?.score[0] : '&#8205;' }}</div>
    </div>
    <div 
      class="flex"
      :class="verticalSeparator"
    >
      <div>{{ !isPaddingMatch ? match?.seeds[1] : '&#8205;' }}</div>
      <div class="grow pl-1">
        {{ match?.players[1] }}
      </div>
      <div>{{ !isPaddingMatch ? match?.score[1] : '&#8205;' }}</div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue';


interface Match {
  id: string,
  players: string[],
  seeds: number[],
  score: number[],
  row_hint: number | null,
}

const props = defineProps<{
    match: Match | null,
}>()

const isPaddingMatch = computed(() => {
  return props.match?.row_hint == null && props.match?.seeds[0] == 0
})

const matchClass = computed(() => {
   if (isPaddingMatch.value) {
    return null
  } else {
    let r = 'max-w-[140px] flex flex-col divide-y border-1 border-box border hover:bg-gray-300 rounded-md'
    r = r + ' ' + `${props.match?.row_hint != null ? `row-start-${props.match.row_hint + 1}` : ''}`
    return r
  }
})

const verticalSeparator = computed(() => {
  if (isPaddingMatch.value) {
    return null
  } else {
    return 'divide-x'
  }
})
</script>
<style scoped>
.match {
  max-width: 200px;
}
</style>