<template>
  <RouterLink
    :to="to"
    class="bg-emerald-400 hover:bg-emerald-500 rounded px-1 py-1"
  >
    <slot />
  </RouterLink>
</template>
<script setup lang="ts">
import { ref } from "vue";

const text = ref("");
const to = ref("");
</script>
