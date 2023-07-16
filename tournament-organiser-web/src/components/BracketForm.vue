<template>
  <input
    v-model="name"
    placeholder="Weekly #"
    class="px-2"
  >
  <SubmitBtn @click="createBracket" />
</template>
<script setup lang="ts">
import { ref } from 'vue'
import SubmitBtn from './SubmitBtn.vue';

const name = ref('')

const createBracket = () => {
    console.log('TODO implement')
}
</script>