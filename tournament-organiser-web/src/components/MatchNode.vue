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
        <div>{{ !isPaddingMatch ? match?.score[0] : '&#8205;' }}</div>
      </div>
      <div 
        class="flex"
        :class="verticalSeparator"
      >
        <div>{{ !isPaddingMatch ? match?.seeds[1] : '&#8205;' }}</div>
        <div class="grow pl-1">
          {{ match?.players[1].name }}
        </div>
        <div>{{ !isPaddingMatch ? match?.score[1] : '&#8205;' }}</div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue';


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
    return 'flex flex-col divide-y border-1 border-box border hover:bg-gray-300 rounded-md'
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