<template>
  <div class="text-lg">
    <slot />
  </div>
  <div
    class="grid grid-rows-1"
    :class="gridClassSetup"
  >
    <div
      v-for="(element, indexCol) in mix"
      :key="indexCol"
      class="grid"
      :class="indexCol < mix.length - 1 ? 'grid-cols-3' : 'grid-cols-1'"
    >
      <div
        class="grid grid-cols-1"
      >
        <MatchNode
          v-for="match in element.match"
          :key="match.id"
          :match="match"
        />
      </div>
      
      <div
        v-if="indexCol < mix.length - 1"
        class="grid grid-cols-1"
      >
        <div
          v-for="(line, index) in element.lines.slice(0, element.lines.length/2)"
          :key="index"
          :class="show(line)"
        />
      </div>
      <div
        v-if="indexCol < mix.length - 1"
        class="grid grid-cols-1"
      >
        <div
          v-for="(line, index) in element.lines.slice(element.lines.length/2, element.lines.length)"
          :key="index"
          :class="show(line)"
        />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { type PropType, computed } from 'vue';
import MatchNode from '@/components/MatchNode.vue';

const props = defineProps({
    bracket: { type: Array as PropType<Match[][]>, default: () => {return []} },
    lines: { type: Array as PropType<Lines[][]>, default: () => {return []} },
})

const mix = computed(() => {
  let lines = props.lines
  lines.push([])
  let r = []
  for (let i = 0; i < props.bracket.length; i++) {
    let o = {
      match: props.bracket[i],
      lines: lines[i],
    }
    r.push(o)
  }

  return r
});

function show(l: Lines) {
  if (l.bottom_border && l.left_border) {
    return 'border-l border-b'
  } else if (l.bottom_border) {
    return 'border-b'
  } else if (l.left_border) {
    return 'border-l'
  } else {
    return ''
  }
}

const gridClassSetup = computed(() => `grid-cols-${props.bracket.length}`)

</script>