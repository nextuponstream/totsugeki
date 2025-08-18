import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ActiveModal = null | 'login'

export const useModalStore = defineStore(
  'modal',
  () => {
    const activeModal = ref<ActiveModal>(null)
    return {
      activeModal,
    }
  },
  {}
)
