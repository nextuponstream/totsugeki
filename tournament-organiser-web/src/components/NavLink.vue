<template>
  <RouterLink :to="to" class="rounded px-1 py-1" :class="navStyle">
    <slot />
  </RouterLink>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps({
  warning: {
    type: Boolean,
    default: () => {
      return false
    },
  },
})
const text = ref('')
const to = ref('')

const navStyle = computed(() => {
  if (props.warning) {
    return 'bg-yellow-400 hover:bg-yellow-500'
  }
  return 'bg-emerald-400 hover:bg-emerald-500'
})
</script>
