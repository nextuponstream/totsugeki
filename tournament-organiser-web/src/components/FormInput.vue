<template>
  <input
    :placeholder="placeholder"
    :value="m"
    class="mt-1 block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring
        focus:ring-indigo-200 focus:ring-opacity-50"
    @input="update"
  >
</template>
<script setup lang="ts">
const props = defineProps({
    placeholder: { type: String, default: ''},
    m: { type: String, default: ''},
})
const emit = defineEmits(['update:m'])

function update($event: any){
    emit('update:m', $event.target.value)
}
</script>