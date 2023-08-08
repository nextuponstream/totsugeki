<template>
  <div class="text-lg">
    <slot />
  </div>
  <div class="flex flex-row overflow-auto overflow-x-auto flex-shrink-0">
    <div
      class="grid grid-rows-1 auto-rows-[1fr] flex-shrink-0"
      :class="gridClassSetup"
    >
      <div
        v-for="(element, indexCol) in mix"
        :key="indexCol"
        class="grid grid-cols-[200px_50px_50px] w-[300px]"
      >
        <div
          class="grid grid-cols-1 w-[200px]"
        >
          <MatchNode
            v-for="match in element.match"
            :key="match.id"
            :match="match"
            @click="showResultModal(match.id, match.players)"
          />
        </div>
        <div
          class="grid grid-cols-1 w-[50px]"
        >
          <div
            v-for="(line, index) in element.lines.slice(0, element.lines.length/2)"
            :key="index"
            :class="show(line)"
          />
        </div>
        <div
          class="grid grid-cols-1 w-[50px]"
        >
          <div
            v-for="(line, index) in element.lines.slice(element.lines.length/2, element.lines.length)"
            :key="index"
            :class="show(line)"
          />
        </div>
      </div>
    </div>
    <div
      v-if="bracketFinalMatch"
      class="my-auto py-auto w-[200px] flex-shrink-0"
    >
      <MatchNode
        :match="bracketFinalMatch"
        @click="showResultModal(bracketFinalMatch.id, bracketFinalMatch.players)"
      />
    </div>
    <div
      v-if="grandFinals"
      class="grid grid-cols-[50px_50px] flex-shrink-0"
    >
      <div class="my-auto">
        <div class="border-b" />
        <div class="" />
      </div>
      <div class="my-auto">
        <div class="border-b" />
        <div class="" />
      </div>
    </div>
    <div
      v-if="grandFinals"
      class="my-auto py-auto grow-0 w-[200px] flex-shrink-0"
    >
      <MatchNode
        :match="grandFinals"
        @click="showResultModal(grandFinals.id, grandFinals.players)"
      />
    </div>
    <div
      v-if="grandFinalsReset"
      class="grid grid-cols-[50px_50px] flex-shrink-0"
    >
      <div class="my-auto">
        <div class="border-b" />
        <div class="" />
      </div>
      <div class="my-auto">
        <div class="border-b" />
        <div class="" />
      </div>
    </div>
    <div
      v-if="grandFinalsReset"
      class="my-auto py-auto grow-0 w-[200px] flex-shrink-0"
    >
      <MatchNode
        :match="grandFinalsReset"
        @click="showResultModal(grandFinalsReset.id, grandFinalsReset.players)"
      />
    </div>
  </div>
</template>
<script setup lang="ts">
import { type PropType, computed } from 'vue';
import MatchNode from '@/components/MatchNode.vue';

const props = defineProps({
    bracket: { type: Array as PropType<Match[][]>, default: () => {return []} },
    lines: { type: Array as PropType<Lines[][]>, default: () => {return []} },
    grandFinals: { type: Object as PropType<Match | undefined>, default: () => {return undefined} },
    grandFinalsReset: { type: Object as PropType<Match | undefined>, default: () => {return undefined} },
})

const emits = defineEmits(['showResultModal'])

const mix = computed(() => {
  let lines = props.lines
  lines.push([])
  let r = []
  for (let i = 0; i < props.bracket.length - 1; i++) {
    let o = {
      match: props.bracket[i],
      lines: lines[i],
    }
    r.push(o)
  }

  return r
});

const bracketFinalMatch = computed(() => {
  let bracket = props.bracket
  if (bracket.length > 0) {
    return bracket[bracket.length - 1][0]
  } else {
    return undefined
  }
})

const gridClassSetup = computed(() => `grid-cols-${props.bracket.length - 1}`)

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

function showResultModal(matchId: string, players: {name: string, id: string}[]){
  emits('showResultModal', matchId, players)
}

</script>