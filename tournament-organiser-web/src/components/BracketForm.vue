<template>
  <div>
    <div class="text-2xl">
      {{ t('bracketForm.title') }}
    </div>
    <div class="pt-2">
      <label>{{ t('bracketForm.nameLabel') }}</label>
      <FormInput
        v-model="bracketName"
        :placeholder="t('bracketForm.namePlaceholder')"
        @keydown.enter="$emit('newBracket', bracketName)"
      />
    </div>
    <div class="pt-2">
      <SubmitBtn
        :disabled="bracketName.length === 0"
        @click="$emit('newBracket', bracketName)"
      />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

const bracketName = ref('')

const emit = defineEmits(['newBracket'])
</script>