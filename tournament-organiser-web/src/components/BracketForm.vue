<template>
  <div>{{ t('app') }}</div>
  <div class="text-xl">
    Create a new bracket
  </div>
  <input
    v-model="name"
    placeholder="Weekly #"
    class="px-2"
  >
  <SubmitBtn @click="$emit('newBracket', name)" />
</template>
<script setup lang="ts">
import { ref } from 'vue'
// @ts-ignore TODO fix lint error
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

const name = ref('')

const emit = defineEmits(['newBracket'])
</script>