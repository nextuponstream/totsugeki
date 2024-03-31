<template>
  <div
    class="grid grid-cols-2 items-center px-2 py-4 bg-emerald-700 mb-3"
    data-test-id="navbar"
  >
    <div class="flex gap-2 items-center">
      <NavLink @click="toggleMenu">
        <i class="pi pi-bars"></i>
      </NavLink>
      <NavLink to="/" text="Home" />
    </div>
    <div class="flex gap-2 items-center justify-self-end">
      <SelectLanguage v-if="!showMenu"></SelectLanguage>
      <NavLink
        v-if="!showMenu"
        to="/about"
        data-test-id="about"
        class="hidden sm:block"
      >
        {{ $t('generic.about') }}
      </NavLink>
      <NavLink
        v-if="userStore.id && !showMenu"
        to="/user/dashboard"
        class="hidden sm:block"
        >{{ $t('generic.profile') }}
      </NavLink>
      <RegisterLogin v-if="!showMenu"></RegisterLogin>
    </div>
  </div>
  <UnsavedBracketModal v-model="unsavedBracketModal"></UnsavedBracketModal>
</template>
<script setup lang="ts">
import NavLink from './NavLink.vue'
import { inject, ref } from 'vue'
import UnsavedBracketModal from './UnsavedBracketModal.vue'
import { useUserStore } from '@/stores/user'
import { showMenuKey } from '@/config'
import SelectLanguage from '@/components/ui/SelectLanguage.vue'
import RegisterLogin from '@/components/ui/RegisterLogin.vue'

const userStore = useUserStore()
const emits = defineEmits(['toggleMenu'])
const showMenu = inject(showMenuKey)

const unsavedBracketModal = ref(false)

function toggleMenu() {
  emits('toggleMenu')
}
</script>
