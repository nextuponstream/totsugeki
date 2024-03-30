<template>
  <ToastZone></ToastZone>
  <div class="min-h-full">
    <side-bar @toggle-menu="toggleMenu"></side-bar>
    <div class="grow">
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
import { provide, type Ref, ref } from 'vue'
import SideBar from '@/components/SideBar.vue'
import { showMenuKey } from '@/config'

const showMenu = ref<boolean | null>(null)
provide<Ref<boolean | null>>(showMenuKey, showMenu)

function toggleMenu() {
  showMenu.value = !showMenu.value
}
</script>
