<template>
  <div class="grow bg-neutral-700 min-h-full">
    <div class="grid grid-cols-1 gap-2 py-3 px-3 text-sm">
      <i
        @click="toggleMenu"
        class="pi pi-times text-gray-400 hover:text-gray-700 text-end"
      ></i>
      <NavLink v-if="userStore.id" @click="logout"
        >{{ $t('generic.logout') }}
      </NavLink>
      <NavLink v-else data-test-id="register" @click="showRegistrationModal">
        {{ $t('generic.registerLogin') }}
        <i class="pi pi-user" />
      </NavLink>
      <NavLink data-test-id="your-brackets"
        >{{ $t('navbar.myBrackets') }}
      </NavLink>
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
      <NavLink v-if="userStore.id" to="/user/dashboard"
        >{{ $t('generic.profile') }}
      </NavLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import NavLink from '@/components/NavLink.vue'
import { useUserStore } from '@/stores/user'
import { useI18n } from 'vue-i18n'
import router from '@/router'
import { inject, ref } from 'vue'
import { useToastStore } from '@/stores/toast'

const userStore = useUserStore()
const toastStore = useToastStore()
const { locale } = useI18n({})
const { t } = useI18n({})
const emits = defineEmits(['toggleMenu'])

function changeLocale(value: any) {
  locale.value = value.target.value
}

async function logout() {
  await userStore.logout()
  toastStore.success(t('logout'))
  await router.push({
    name: 'createBracket',
  })
}

// FIXME login does not work from the sidebar
function showRegistrationModal() {
  registrationModal.value = true
}

const supportedLocales = ['en', 'fr']
const registrationModal = ref(false)

function toggleMenu() {
  emits('toggleMenu')
}
</script>

<style scoped></style>
