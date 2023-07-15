<template>
  <input
    v-model="name"
    placeholder="Weekly #"
  >
  <button @click="createBracket">
    Create bracket
  </button>
</template>
<script setup lang="ts">
import { ref } from 'vue'

const name = ref('')

const createBracket = () => {
    console.log('TODO implement')
}
</script>