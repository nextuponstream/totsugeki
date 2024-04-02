<template>
  <BaseModal v-model="showModal" title="TODO" prefix="TODO"></BaseModal>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import BaseModal from './ui/modals/BaseModal.vue'

const showModal = ref(false)
</script>
