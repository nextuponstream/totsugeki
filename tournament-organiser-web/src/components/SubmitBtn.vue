<template>
  <button class=" bg-green-700 hover:bg-green-800 rounded-lg py-2 px-2 text-white">
    {{ t('generic.submit') }}
  </button>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})
</script>