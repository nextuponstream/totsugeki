<template>
  <div class="flex bg-emerald-900 py-3 px-3 gap-2">
    <NavLink
      to="/"
      text="Home"
    />
    <NavLink
      to="/about"
      text="About"
      data-test-id="about"
    />
    <select @input="changeLocale">
      <option value="en">
        EN
      </option>
      <option value="fr">
        FR
      </option>
    </select>
  </div>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import NavLink from './NavLink.vue';

const {locale} = useI18n({})

function changeLocale(value: any) {
  locale.value = value.target.value
}
</script>