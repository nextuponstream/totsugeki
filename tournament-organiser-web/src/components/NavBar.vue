<template>
  <div class="flex flex-wrap items-center justify-between px-2 py-2 bg-emerald-700 mb-3">
    <NavLink
      to="/"
      text="Home"
    />
    <div class="flex gap-2 items-center">
      <div class="py-1">
        <NavLink
          to="/about"
          text="About"
          data-test-id="about"
        />
      </div>
      <div class="flex">
        <select
          class="mt-1 block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring
        focus:ring-indigo-200 focus:ring-opacity-50"
          @input="changeLocale"
        >
          <option
            v-for="supportedLocale in supportedLocales"
            :key="supportedLocale"
            :value="supportedLocale"
          >
            {{ supportedLocale }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import NavLink from './NavLink.vue';

const supportedLocales = [
  'en',
  'fr',
]

const {locale} = useI18n({})

function changeLocale(value: any) {
  locale.value = value.target.value
}
</script>