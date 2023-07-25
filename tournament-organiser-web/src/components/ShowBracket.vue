<template>
  <div class="text-lg">
    <slot />
  </div>
  <div
    class="grid grid-rows-1"
    :class="`grid-cols-${bracket.length}`"
  >
    <div
      v-for="(element, index) in bracket"
      :key="index"
    >
      <div
        v-for="match in element"
        :key="match.id"
        class="grid grid-cols-1"
      >
        <MatchNode :match="match" />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { type PropType } from 'vue';
import MatchNode from '@/components/MatchNode.vue';

const props = defineProps({
    bracket: { type: Array as PropType<Match[][]>, default: () => {return []} }
})
</script>