<template>
  <form class="flex flex-col max-w-xs gap-3" name="user-edit">
    <label>{{ $t('generic.email') }}</label>
    <FormInput name="email" type="email" v-bind="emailAttrs" :disabled="true" />
    <label>{{ $t('generic.username') }}</label>
    <FormInput name="name" type="text" v-bind="nameAttrs" :disabled="true" />
  </form>
</template>
<script setup lang="ts">
import { useForm } from 'vee-validate'
import { object, string } from 'yup'
import { useI18n } from 'vue-i18n'
import { watch } from 'vue'
const { t } = useI18n({})

const editSchema = object({
  email: string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
  password: string()
    .required(() => t('error.required'))
    .min(8, () => t('error.minimum', { min: 8 })),
})
const { defineField, setValues } = useForm({
  validationSchema: editSchema,
})

defineExpose({ setValues })

// User info form
const [email, emailAttrs] = defineField('email')
const [name, nameAttrs] = defineField('name')
</script>
