<template>
  <NavLink v-if="userStore.id" @click="logout"
    >{{ $t('generic.logout') }}
  </NavLink>
  <NavLink v-else data-test-id="register" @click="showRegistrationModal">
    {{ $t('generic.registerLogin') }}
    <i class="pi pi-user" />
  </NavLink>
</template>

<script setup lang="ts">
import NavLink from '@/components/NavLink.vue'

import { useUserStore } from '@/stores/user'
import router from '@/router'
import { useI18n } from 'vue-i18n'
import { useToastStore } from '@/stores/toast'
import { useModalStore } from '@/stores/modal'

const { t } = useI18n({})
const toastStore = useToastStore()
const modalStore = useModalStore()
const userStore = useUserStore()

function showRegistrationModal() {
  modalStore.activeModal = 'login'
}

async function logout() {
  await userStore.logout()
  toastStore.success(t('logout'))
  await router.push({
    name: 'createBracket',
  })
}
</script>
