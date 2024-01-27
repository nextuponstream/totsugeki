<template>
  <a :href="href">
    <slot />
  </a>
</template>
<script setup lang="ts">
import { ref } from 'vue';

const href = ref('')
</script>
<style scoped>
a {
    color: blue;
}

a:hover {
    color: lightblue;
}
</style>