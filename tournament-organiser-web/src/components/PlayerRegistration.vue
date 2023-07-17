<template>
  <div class="text-xl">
    Register new player
  </div>
  <input
    v-model="name"
    placeholder="Player name"
    class="px-2"
  >
  <SubmitBtn @click="$emit('newPlayer', name)" />
</template>
<script setup lang="ts">
import { ref } from 'vue'

const name = ref('')

const emit = defineEmits(['newPlayer'])
</script>