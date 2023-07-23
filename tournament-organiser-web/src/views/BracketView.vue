<template>
  {{ bracket.winner_bracket }}
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue';

const bracket = ref({
  winner_bracket: true
})

onMounted(() => {
  bracket.value = JSON.parse(localStorage.getItem('bracket')!)
})

</script>