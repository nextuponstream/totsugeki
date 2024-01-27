<template>
  <div class="py-2 flex flex-col" :class="rowClass">
    <div class="grid grid-rows-2 flex-col my-auto" :class="matchClass">
      <div class="grid grid-cols-7">
        <div
          class="text-xs text-center py-1"
          :class="rowDividerAndVerticalSeparator"
        >
          {{ !isPaddingMatch ? showSeed(match?.seeds[0]) : "&#8205;" }}
        </div>
        <div
          class="col-span-5 text-xs py-1 px-1"
          :class="rowDividerAndVerticalSeparator"
        >
          {{ match?.players[0].name }}
        </div>
        <MatchScore
          :is-padding-match="isPaddingMatch"
          :scores="match?.score"
          :index="0"
          :other-index="1"
          :class="rowDivider"
        />
      </div>
      <div class="grid grid-cols-7">
        <div class="text-xs text-center py-1" :class="verticalSeparator">
          {{ !isPaddingMatch ? showSeed(match?.seeds[1]) : "&#8205;" }}
        </div>
        <div class="col-span-5 text-xs py-1 px-1" :class="verticalSeparator">
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
import { computed } from "vue";
import MatchScore from "./match/MatchScore.vue";

// TODO hover only for matches for which you can submit scores
// FIXME nitpick lines do not flow pixel perfectly into/out of matches https://github.com/nextuponstream/totsugeki/issues/38

interface Match {
  id: string;
  players: { name: string; id: string }[];
  seeds: number[];
  score: number[];
  row_hint: number | null;
}

const props = defineProps<{
  match: Match;
}>();

const isPaddingMatch = computed(() => {
  return props.match?.row_hint == null && props.match?.seeds[0] == 0;
});

const matchClass = computed(() => {
  if (isPaddingMatch.value) {
    return null;
  } else {
    return "flex flex-col border-box border hover:bg-gray-300";
  }
});

const verticalSeparator = computed(() => {
  if (isPaddingMatch.value) {
    return null;
  } else {
    return "border-r";
  }
});

const rowDividerAndVerticalSeparator = computed(() => {
  if (isPaddingMatch.value) {
    return null;
  } else {
    return "border-b border-r";
  }
});

const rowDivider = computed(() => {
  if (isPaddingMatch.value) {
    return null;
  } else {
    return "border-b";
  }
});

const totalPlayers = 3;

const rowClass = computed(() => {
  return `${
    props.match?.row_hint != null ? `row-start-${props.match.row_hint + 1}` : ""
  }`;
});

/**
 * Pad seed with white spaces
 * @param seed
 */
function showSeed(seed: number) {
  let s = seed.toString();
  return s;
}
</script>
