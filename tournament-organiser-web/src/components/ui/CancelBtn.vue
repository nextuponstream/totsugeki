<template>
  <button class="bg-gray-500 hover:bg-gray-600 rounded-lg py-2 px-2 text-white">
    <slot>{{ t("generic.cancel") }}</slot>
  </button>
</template>
<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n({});
</script>
