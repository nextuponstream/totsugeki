<template>
  <ToastZone></ToastZone>
  <div class="grid min-h-full" :class="pageLayout">
    <div v-if="showMenu">
      <side-bar @toggle-menu="toggleMenu"></side-bar>
    </div>
    <div class="sm:col-span-4 col-span-1">
      <NavBar @toggle-menu="toggleMenu" />
      <div class="px-9 py-9">
        <RouterView />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { RouterView } from 'vue-router'
import NavBar from './components/NavBar.vue'
import ToastZone from './components/ToastZone.vue'
import { computed, provide, ref } from 'vue'
import SideBar from '@/components/SideBar.vue'

const showMenu = ref(false)
provide('showMenu', showMenu)

function toggleMenu() {
  showMenu.value = !showMenu.value
}

const pageLayout = computed(() => {
  return showMenu.value ? 'sm:grid-cols-5 grid-cols-2' : 'grid-cols-1'
})
</script>
