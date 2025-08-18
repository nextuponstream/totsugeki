<template>
  <span
    class="absolute mx-2 -my-2 px-2 py-2 text-gray-600 bg-gray-300 group-hover:opacity-100 transition-opacity text-sm rounded-md opacity-0"
  >
    <slot />
  </span>
</template>
<script setup lang="ts">
// NOTES:
// * parent must implement class relative+group
// * tooltip appears on the right side
// TODO box containing tooltip is not right size
</script>
