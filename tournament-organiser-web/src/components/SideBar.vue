<template>
  <div
    class="fixed min-h-full overflow-hidden grow bg-neutral-700 h-full"
    :class="openedSidebar"
  >
    <div class="grid grid-cols-1 gap-2 py-3 px-3 text-sm">
      <i
        class="pi pi-times text-gray-400 hover:text-gray-700 text-end"
        @click="toggleMenu"
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
      <SelectLanguage></SelectLanguage>
      <NavLink to="/about" data-test-id="about">
        {{ $t('generic.about') }}
      </NavLink>
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
import { computed, inject, ref } from 'vue'
import { useToastStore } from '@/stores/toast'
import { showMenuKey } from '@/config'
import SelectLanguage from '@/components/ui/SelectLanguage.vue'

const userStore = useUserStore()
const toastStore = useToastStore()
const { t } = useI18n({})
const emits = defineEmits(['toggleMenu'])
const showMenu = inject(showMenuKey)

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

const registrationModal = ref<Boolean | null>(null)

function toggleMenu() {
  emits('toggleMenu')
}

const openedSidebar = computed(() => {
  console.info(showMenu?.value)
  if (showMenu !== undefined) {
    if (showMenu.value === null) {
      return 'sidebar'
    }
    return showMenu.value ? 'sidebar-opened' : 'sidebar-closed'
  }
  return ''
})
</script>

<style scoped>
.sidebar {
  left: -500px;
}

.sidebar-opened {
  animation: slide-in 0.5s forwards;
  left: -500px;
}

.sidebar-closed {
  animation: slide-out 0.5s forwards;
  left: -500px;
}

@keyframes slide-in {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(500px);
  }
}

@keyframes slide-out {
  0% {
    transform: translateX(500px);
  }
  100% {
    transform: translateX(0px);
  }
}
</style>
