<template>
  <div class="flex flex-wrap items-center justify-between px-2 py-2 bg-emerald-700 mb-3">
    <NavLink to="/" text="Home" />
    <div class="flex gap-2 items-center">
      <select
        class="my-1 px-1 py-1 block rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
        @input="changeLocale">
        <option v-for="supportedLocale in supportedLocales" :key="supportedLocale" :value="supportedLocale">
          {{ supportedLocale }}
        </option>
      </select>
      <NavLink to="/about" data-test-id="about">
        {{ $t("generic.about") }}
      </NavLink>
      <NavLink data-test-id="register" @click="showRegistrationModal">
        {{ $t("generic.registerLogin") }}
        <i class="pi pi-user" />
      </NavLink>
    </div>
  </div>
  <UserRegistrationModal v-model="registrationModal"></UserRegistrationModal>
</template>
<script setup lang="ts">
import { useI18n } from "vue-i18n";
import NavLink from "./NavLink.vue";
import { ref } from 'vue';
import UserRegistrationModal from "./UserRegistrationModal.vue";

const supportedLocales = ["en", "fr"];

const { locale } = useI18n({});
const registrationModal = ref(false)

function changeLocale(value: any) {
  locale.value = value.target.value;
}

function showRegistrationModal() {
  registrationModal.value = true
}
</script>
