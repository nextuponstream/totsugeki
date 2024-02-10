<template>
  <div
    class="text-xs place-self-stretch grid"
    :class="scoreClass(scores, index, otherIndex)"
  >
    <div class="place-self-center">
      {{ !isPaddingMatch ? scores[index] : '&#8205;' }}
    </div>
  </div>
</template>
<script setup lang="ts">
const props = defineProps<{
  isPaddingMatch: boolean
  scores: number[]
  index: number
  otherIndex: number
}>()

function scoreClass(
  scores: number[] | undefined,
  index: number,
  otherIndex: number
) {
  if (scores?.length === 2 && scores[index] > scores[otherIndex]) {
    return 'text-white bg-green-600'
  } else {
    return ''
  }
}
</script>
