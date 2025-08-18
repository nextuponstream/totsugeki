<template>
  <div
    class="toast text-sm transition-opacity duration-1000 ease-in-out opacity-0"
    :class="toastClasses"
  >
    <div v-if="title">{{ title }}</div>
    <div class="font-light">{{ text }}</div>
  </div>
</template>
<script setup lang="ts">
import type { ToastStatus } from '@/stores/toast'
import { onMounted, type PropType, reactive } from 'vue'

const emits = defineEmits(['close-toast'])

const props = defineProps({
  title: {
    type: String,
    required: false,
    default: () => {
      return undefined
    },
  },
  text: {
    type: String,
    required: true,
  },
  type: {
    type: String as PropType<ToastStatus>,
    required: true,
  },
})

const toastClasses = reactive([] as string[])

onMounted(() => {
  switch (props.type) {
    case 'success':
      toastClasses.push('bg-emerald-400')
      break
    case 'warning':
      toastClasses.push('bg-yellow-400')
      break
    case 'error':
      toastClasses.push('bg-red-400')
      break
    default:
      break
  }

  setTimeout(() => {
    toastClasses.push('opacity-100')
  }, 1)
  setTimeout(() => {
    toastClasses.pop()
    toastClasses.push('opacity-0')
  }, 3000)
})
</script>
<style scoped>
.toast {
  border-radius: 1em;
  padding: 1em;
  min-width: 150px;
  min-height: 60px;
}
</style>
