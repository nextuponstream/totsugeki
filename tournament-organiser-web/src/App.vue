<template>
  <NavBar v-model="user" />
  <div class="px-9 py-9">
    <RouterView @logout="refreshNavbar()" />
  </div>
</template>
<script setup lang="ts">
import { RouterView } from 'vue-router'
import NavBar from './components/NavBar.vue'
import { ref } from 'vue'

const user = ref(false)

function refreshNavbar() {
  user.value = false
}
</script>
