<template>
  <div>
    <div class="text-xl">
      {{ t('bracketForm.title') }}
    </div>
    <div class="pt-2">
      <label>{{ t('bracketForm.nameLabel') }}</label>
      <FormInput
        v-model="name"
        :placeholder="t('bracketForm.namePlaceholder')"
        @keydown.enter="$emit('newBracket', name)"
      />
    </div>
    <div class="pt-2">
      <SubmitBtn @click="$emit('newBracket', name)" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n';

const {t} = useI18n({})

const name = ref('')

const emit = defineEmits(['newBracket'])
</script>