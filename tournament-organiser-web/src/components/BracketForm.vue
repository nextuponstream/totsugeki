<template>
  <div class="text-xl">
    Create a new bracket
  </div>
  <input
    v-model="name"
    placeholder="Weekly #"
    class="px-2"
  >
  <SubmitBtn @click="$emit('newBracket', name)" />
</template>
<script setup lang="ts">
import { ref } from 'vue'
import SubmitBtn from './SubmitBtn.vue';

const name = ref('')

const emit = defineEmits(['newBracket'])
</script>