<template>
  <button class="bg-gray-500 hover:bg-gray-600 rounded-lg py-2 px-2 text-white">
    <slot>{{ $t("generic.cancel") }}</slot>
  </button>
</template>
<script setup lang="ts">
// NOTE sometimes hover effect does not work in development. Then just close the browser and reopen it
</script>
