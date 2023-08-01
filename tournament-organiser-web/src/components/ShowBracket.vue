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
    >
      <div
        v-for="(match, index) in element"
        :key="match.id"
      >
        <MatchNode
          :match="match"
          :class="`row-start-${index+1}`"
        />
      </div>
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