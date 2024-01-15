<template>
  <input
    :placeholder="placeholder"
    :value="modelValue"
    class="block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring
        focus:ring-indigo-200 focus:ring-opacity-50 invalid:border-red-500"
    invalid
    @input="$event => $emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  >
</template>
<script setup lang="ts">

const props = defineProps({
    placeholder: { type: String, default: ''},
    modelValue: { type: String, default: ''},
})
const emit = defineEmits(['update:modelValue'])
</script>