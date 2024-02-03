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
      <NavLink v-if="isLogged" to="/user/dashboard">{{
        $t('generic.profile')
      }}</NavLink>
      <NavLink v-if="isLogged" @click="logout">{{
        $t('generic.logout')
      }}</NavLink>
      <NavLink v-else data-test-id="register" @click="showRegistrationModal">
        {{ $t('generic.registerLogin') }}
        <i class="pi pi-user" />
      </NavLink>
    </div>
  </div>
  <UserLoginModal v-model="registrationModal" @login="login" />
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import NavLink from './NavLink.vue'
import { onMounted, ref } from 'vue'
import UserLoginModal from './UserLoginModal.vue'
import router from '@/router'

const supportedLocales = ['en', 'fr']

const { locale } = useI18n({})
const registrationModal = ref(false)

const isLogged = ref(false)

onMounted(() => {
  refresh()
})

function refresh() {
  let userId = localStorage.getItem('user_id')
  isLogged.value = userId !== null
}

function changeLocale(value: any) {
  locale.value = value.target.value
}

function showRegistrationModal() {
  registrationModal.value = true
}

function login() {
  registrationModal.value = false
  refresh()
}

async function logout() {
  try {
    let response = await fetch(`${import.meta.env.VITE_API_URL}/api/logout`, {
      method: 'POST',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
      credentials: 'same-origin',
    })
    if (response.ok) {
      console.info('successful logout')
      localStorage.removeItem('user_id')
      refresh()
      router.push({
        name: 'createBracket',
      })
    } else {
      throw new Error('non-200 response for /api/logout')
    }
  } catch (e) {
    console.error(e)
  }
}
</script>
