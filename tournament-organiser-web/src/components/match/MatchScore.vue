<template>
  <div
    class="px-1 py-1 text-xs" 
    :class="scoreClass(scores, index, otherIndex)"
  >
    {{ !isPaddingMatch ? scores[index] : '&#8205;' }}
  </div>
</template>
<script setup lang="ts">

const props = defineProps<{
    isPaddingMatch: boolean,
    scores: number[],
    index: number,
    otherIndex: number,
}>()

function scoreClass(scores: number[] | undefined, index: number, otherIndex: number) {
  if (scores?.length == 2 && scores[index] > scores[otherIndex]) {
    return 'text-white bg-green-600'
  } else {
    return ''
  }
}
</script>