<template>
  <input
    :placeholder="placeholder"
    :value="modelValue"
    class="mt-1 block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring
        focus:ring-indigo-200 focus:ring-opacity-50"
    @input="$event => $emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  >
</template>
<script setup lang="ts">

const props = defineProps({
    placeholder: { type: String, default: ''},
    modelValue: { type: String, default: ''},
})
const emit = defineEmits(['update:modelValue'])

// moved outside of vue template because typescript complains about $event being possible null
/* function update($event: any) {
  emit('update:modelValue', $event.target.value)
} */
</script>