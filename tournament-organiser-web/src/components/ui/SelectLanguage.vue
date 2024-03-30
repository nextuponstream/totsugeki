<template>
  <select
    v-model="languageStore.language"
    class="px-1 py-1 block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
    autocomplete="off"
    @input="changeLocale"
  >
    <option
      v-for="supportedLocale in languageStore.supportedLanguages"
      :key="supportedLocale"
    >
      {{ supportedLocale }}
    </option>
  </select>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useLanguageStore } from '@/stores/language'

const languageStore = useLanguageStore()
const { locale } = useI18n({})

function changeLocale(value: any) {
  locale.value = value.target.value
}
</script>
