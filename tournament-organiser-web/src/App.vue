<template>
  <NavBar />
  <div class="px-4">
    <RouterView />
  </div>
</template>
<script setup lang="ts">
import { RouterView } from 'vue-router'
import NavBar from './components/NavBar.vue';
</script>