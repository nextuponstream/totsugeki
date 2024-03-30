<template>
  <div class="grid grid-cols-2 items-center px-2 py-4 bg-emerald-700 mb-3">
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
      <NavLink v-if="userStore.id && !showMenu" @click="logout"
        >{{ $t('generic.logout') }}
      </NavLink>
      <NavLink
        v-else-if="!showMenu"
        data-test-id="register"
        @click="showRegistrationModal"
      >
        {{ $t('generic.registerLogin') }}
        <i class="pi pi-user" />
      </NavLink>
    </div>
  </div>
  <UserLoginModal v-model="registrationModal" @login="login" />
  <UnsavedBracketModal v-model="unsavedBracketModal"></UnsavedBracketModal>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import NavLink from './NavLink.vue'
import { inject, ref } from 'vue'
import UserLoginModal from './UserLoginModal.vue'
import UnsavedBracketModal from './UnsavedBracketModal.vue'
import { useUserStore } from '@/stores/user'
import router from '@/router'
import { useToastStore } from '@/stores/toast'
import { showMenuKey } from '@/config'
import SelectLanguage from '@/components/ui/SelectLanguage.vue'

const { t } = useI18n({})
const userStore = useUserStore()
const toastStore = useToastStore()
const emits = defineEmits(['toggleMenu'])
const showMenu = inject(showMenuKey)

const registrationModal = ref(false)
const unsavedBracketModal = ref(false)

function showRegistrationModal() {
  registrationModal.value = true
}

function login() {
  registrationModal.value = false
}

async function logout() {
  await userStore.logout()
  toastStore.success(t('logout'))
  await router.push({
    name: 'createBracket',
  })
}

function toggleMenu() {
  emits('toggleMenu')
}
</script>
