<template>
  <div class="text-lg">
    <slot />
  </div>
  <div
    class="grid grid-rows-1 flex"
    :class="gridClassSetup"
  >
    <div
      v-for="(element, indexCol) in bracket"
      :key="indexCol"
      class="grid grid-cols-1"
    >
      <MatchNode
        v-for="match in element"
        :key="match.id"
        class="my-auto"
        :match="match"
      />
    </div>
  </div>
</template>
<script setup lang="ts">
import { type PropType, computed } from 'vue';
import MatchNode from '@/components/MatchNode.vue';

const props = defineProps({
    bracket: { type: Array as PropType<Match[][]>, default: () => {return []} }
})

const gridClassSetup = computed(() => `grid-cols-${props.bracket.length}`)

</script>