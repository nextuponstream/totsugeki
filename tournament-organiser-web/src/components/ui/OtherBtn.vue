<template>
  <button
    class="bg-blue-700 hover:bg-blue-800 rounded-lg py-2 px-2 text-white disabled:bg-gray-400"
  >
    <slot />
  </button>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n({})
</script>
<style scoped></style>
