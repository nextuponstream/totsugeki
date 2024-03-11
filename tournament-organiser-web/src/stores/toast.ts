// NOTE: trying vue-toastification and trying to change the appearance was a
// pain. I am not installing vue-bootstrap for toast. Other library look
// unpopular. Then let's use that article from medium
// https://medium.com/@serpentarium13/how-to-toast-on-your-own-in-vue-5962c0f954d1

import { defineStore } from 'pinia'
import { type Ref, ref } from 'vue'
export type ToastStatus = 'success' | 'warning' | 'error'
export interface Toast {
  id: number
  title?: string
  content: string
  timeout?: number
  type: ToastStatus
}

const defaultTimeout = 3000

export const useToastStore = defineStore('toast', () => {
  const toasts: Ref<Array<Toast>> = ref([])

  function makeToast(
    content: string,
    type: ToastStatus,
    timeout?: number,
    title?: string
  ) {
    let id = Date.now()
    console.debug('new toast')
    toasts.value.push({
      id,
      title,
      content,
      type,
      timeout: timeout ?? defaultTimeout,
    })
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id)
    }, timeout ?? defaultTimeout)
  }

  function success(content: string, timeout?: number, title?: string) {
    makeToast(content, 'success', timeout, title)
  }

  function error(content: string, timeout?: number, title?: string) {
    makeToast(content, 'error', timeout, title)
  }

  return {
    toasts,
    success,
    error,
  }
})
