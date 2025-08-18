<template>
  <div
    class="fixed z-40 w-screen h-screen inset-0 bg-gray-900 bg-opacity-60"
    :data-test-id="backgroundName"
  />
</template>
<script setup lang="ts">
import { computed, inject } from 'vue'
import { prefixKey } from '@/config'

const prefix = inject(prefixKey)
const backgroundName = computed(() => {
  return `${prefix}-blurred-background-outside-modal`
})
</script>
