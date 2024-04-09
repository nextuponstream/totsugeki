<template>
  <blurred-background :class="isHidden" @click="toggleMenu" />
  <div
    class="fixed min-h-full overflow-hidden grow bg-neutral-700 h-full z-50"
    :class="openedSidebar"
  >
    <div class="grid grid-cols-1 gap-2 py-3 px-3 text-sm">
      <i
        class="pi pi-times text-gray-400 hover:text-gray-700 text-end"
        @click="toggleMenu"
      ></i>
      <NavLink to="/" text="Home" />
      <RegisterLogin></RegisterLogin>
      <NavLink
        v-if="userStore.id"
        data-test-id="my-brackets"
        to="/user/brackets"
        @click="toggleMenu"
        >{{ $t('navbar.myBrackets') }}
      </NavLink>
      <SelectLanguage></SelectLanguage>
      <NavLink to="/about" data-test-id="about" @click="toggleMenu">
        {{ $t('generic.about') }}
      </NavLink>
      <NavLink v-if="userStore.id" to="/user/dashboard" @click="toggleMenu"
        >{{ $t('generic.profile') }}
      </NavLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import NavLink from '@/components/NavLink.vue'
import { useUserStore } from '@/stores/user'
import { computed, inject, provide } from 'vue'
import { prefixKey, showMenuKey } from '@/config'
import SelectLanguage from '@/components/ui/SelectLanguage.vue'
import RegisterLogin from '@/components/ui/RegisterLogin.vue'
import BlurredBackground from '@/components/ui/modals/BlurredBackground.vue'

const userStore = useUserStore()
const emits = defineEmits(['toggleMenu'])
const showMenu = inject(showMenuKey)
provide(prefixKey, 'sidebar')

function toggleMenu() {
  emits('toggleMenu')
}

const openedSidebar = computed(() => {
  if (showMenu !== undefined) {
    if (showMenu.value === null) {
      return 'sidebar'
    }
    return showMenu.value ? 'sidebar-opened' : 'sidebar-closed'
  }
  return ''
})

const isHidden = computed(() => {
  if (showMenu !== undefined) {
    return showMenu.value ? null : 'hidden'
  }

  return 'hidden'
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
