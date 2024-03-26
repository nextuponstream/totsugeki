<template>
  <ToastZone></ToastZone>

  <NavBar />
  <div class="px-9 py-9">
    <RouterView />
  </div>
</template>
<script setup lang="ts">
import { RouterView } from 'vue-router'
import NavBar from './components/NavBar.vue'
import ToastZone from './components/ToastZone.vue'
</script>
