<template>
  <BaseModal v-model="showModal" title="TODO"></BaseModal>
</template>
<script setup lang="ts">
import { provide, ref } from 'vue'
import BaseModal from './ui/modals/BaseModal.vue'
import { prefixKey } from '@/config'

const showModal = ref(false)
provide(prefixKey, 'unsaved-bracket')
</script>
