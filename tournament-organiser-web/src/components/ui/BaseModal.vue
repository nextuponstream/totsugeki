<template>
  <div
    class="fixed z-40 w-screen h-screen inset-0 bg-gray-900 bg-opacity-60"
    :class="isHidden"
    :data-test-id="backgroundName"
    @click="hideModal"
  />
  <div
    class="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-auto max-w-sm bg-white rounded-md px-8 py-6 space-y-5 drop-shadow-lg"
    :class="isHidden"
    data-test-id="modal"
  >
    <div class="flex flex-col gap-1">
      <div class="self-end">
        <button @click="hideModal">
          <i class="pi pi-times p-2 text-gray-400 hover:text-gray-700" />
        </button>
      </div>
      <h1 class="text-2xl font-semibold">
        {{ title }}
      </h1>
      <slot />
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'

const emits = defineEmits(['update:modelValue', 'hide'])

const props = defineProps<{
  modelValue: boolean
  title: string
  prefix: string
}>()

function hideModal() {
  emits('hide')
}

const backgroundName = computed(() => {
  return `${props.prefix}-blurred-background-outside-modal`
})

const isHidden = computed(() => {
  if (props.modelValue) {
    return null
  } else {
    return 'hidden'
  }
})
</script>
