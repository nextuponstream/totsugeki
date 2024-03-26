<template>
  <div class="toast text-sm" :class="toastClass(type)">
    <div v-if="title">{{ title }}</div>
    <div class="font-light">{{ text }}</div>
  </div>
</template>
<script setup lang="ts">
import type { ToastStatus } from '@/stores/toast'
import type { PropType } from 'vue'

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

function toastClass(type: ToastStatus) {
  switch (type) {
    case 'success':
      return 'bg-emerald-400/90'
    case 'warning':
      return 'bg-yellow-400/90'
    case 'error':
      return 'bg-red-400/90'
  }
}
</script>
<style scoped>
.toast {
  border-radius: 1em;
  padding: 1em;
  opacity: 0.9;
  min-width: 150px;
  min-height: 60px;
}
</style>
