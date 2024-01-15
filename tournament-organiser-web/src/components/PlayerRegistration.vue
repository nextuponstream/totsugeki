<template>
  <div class="grid grid-cols-1 gap-2">
    <div class="text-2xl">
      {{ t('playerRegistrationForm.title') }}
    </div>
    <div class="flex gap-2 align-baseline">
      <FormInput
        v-model="name"
        :placeholder="t('playerRegistrationForm.newPlayerPlaceholder')"
        @keydown.enter="submitPlayer"
      />
      <SubmitBtn
        :disabled="name.length === 0"
        @click="submitPlayer"
      />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

const name = ref('')

const emit = defineEmits(['newPlayer'])

function submitPlayer(){
  if (name.value.length > 0) {
    emit('newPlayer', name.value)
    name.value = ''
  }
}
</script>