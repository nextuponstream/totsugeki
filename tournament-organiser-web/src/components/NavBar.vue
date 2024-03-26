<template>
  <div
    class="flex flex-wrap items-center justify-between px-2 py-2 bg-emerald-700 mb-3"
  >
    <NavLink to="/" text="Home" />
    <div class="flex gap-2 items-center">
      <select
        class="my-1 px-1 py-1 block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
        @input="changeLocale"
      >
        <option
          v-for="supportedLocale in supportedLocales"
          :key="supportedLocale"
          :value="supportedLocale"
        >
          {{ supportedLocale }}
        </option>
      </select>
      <NavLink to="/about" data-test-id="about">
        {{ $t('generic.about') }}
      </NavLink>
      <!-- <NavLink @click="test"> succ {{ toastStore.toasts.length }} </NavLink>
      <NavLink @click="test2"> err {{ toastStore.toasts.length }} </NavLink> -->
      <NavLink v-if="userStore.id" to="/user/dashboard">{{
        $t('generic.profile')
      }}</NavLink>
      <NavLink v-if="userStore.id" @click="logout">{{
        $t('generic.logout')
      }}</NavLink>
      <NavLink v-else data-test-id="register" @click="showRegistrationModal">
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
import { ref } from 'vue'
import UserLoginModal from './UserLoginModal.vue'
import UnsavedBracketModal from './UnsavedBracketModal.vue'
import { useUserStore } from '@/stores/user'
import router from '@/router'
import { useToastStore } from '@/stores/toast'
const { t } = useI18n({})
const userStore = useUserStore()

const supportedLocales = ['en', 'fr']

const { locale } = useI18n({})
const registrationModal = ref(false)
const unsavedBracketModal = ref(false)

function changeLocale(value: any) {
  locale.value = value.target.value
}

function showRegistrationModal() {
  registrationModal.value = true
}

function login() {
  registrationModal.value = false
}

async function logout() {
  await userStore.logout()
  toastStore.success(t('logout'))
  router.push({
    name: 'createBracket',
  })
}

const toastStore = useToastStore()
// function test() {
//   toastStore.success('content')
// }
// function test2() {
//   toastStore.error('content')
// }
</script>
