<template>
  <div class="text-2xl">
    {{ t('playerRegistrationForm.title') }}
  </div>
  <div>
    <FormInput
      v-model="name"
      :placeholder="t('playerRegistrationForm.newPlayerPlaceholder')"
      @keydown.enter="submitPlayer"
    />
  </div>
  <div class="pt-2">
    <SubmitBtn
      :disabled="name.length === 0"
      @click="submitPlayer"
    />
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

const name = ref('')

const emit = defineEmits(['newPlayer'])

function submitPlayer(){
  emit('newPlayer', name.value)
  name.value = ''
}
</script>