<template>
  <button
    class="max-w-[200px] max-h-11 bg-red-600 hover:bg-red-900 rounded-lg py-2 px-2 text-white disabled:bg-gray-400"
  >
    <slot />
  </button>
</template>
<script setup lang="ts">
// NOTE: on firefox with mobile view, hover doesn "not work" because how do you
// hover with a mobile device? Exactly https://stackoverflow.com/a/67624464
</script>
<style scoped></style>
