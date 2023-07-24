<template>
  <div class="col-span-1 flex flex-col my-auto divide-x divide-y border-1 border-box border hover:bg-gray-300 rounded-md">
    <div class="match flex flex-row divide-x">
      <div>{{ match?.seeds[0] }}</div>
      <div class="grow pl-1">
        {{ match?.players[0] }}
      </div>
      <div>{{ match?.score[0] }}</div>
    </div>
    <div class="match flex flex-row divide-x">
      <div>{{ match?.seeds[1] }}</div>
      <div class="grow pl-1">
        {{ match?.players[1] }}
      </div>
      <div>{{ match?.score[1] }}</div>
    </div>
  </div>
</template>
<script setup lang="ts">

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


</script>
<style scoped>
.match {
  max-width: 30px;
}
</style>