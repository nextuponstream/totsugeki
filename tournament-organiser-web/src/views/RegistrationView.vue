<template>
  <Form
    class="flex flex-col max-w-xs gap-3"
    :on-submit="onSubmit"
    :on-invalid-submit="onInvalidSubmit"
    :validation-schema="schema"
    autocomplete
    name="user-registration"
  >
    <label>{{ $t('generic.email') }}</label>
    <FormInput name="email" type="email" />
    <label>{{ $t('generic.username') }}</label>
    <FormInput name="name" type="text" />
    <label>{{ $t('generic.password') }}</label>
    <FormInput name="password" type="password" />
    <label>{{ $t('generic.confirmPassword') }}</label>
    <FormInput name="confirmPassword" type="password" />
    <SubmitBtn>{{ $t('generic.register') }}</SubmitBtn>
  </Form>
</template>
<script setup lang="ts">
// TODO when an error is displayed, locale of errors does not update automatically on locale change
// tried looking online but it's a bad interaction between vee-validate and i18n I guess.
// However, it's not a big concern
import { Form } from 'vee-validate'
import { ref, provide } from 'vue'
import { object, string, ref as yupref } from 'yup'
import { useI18n } from 'vue-i18n'
import router from '@/router'

const { t } = useI18n({})
// NOTE: how to use i18n with yup https://stackoverflow.com/questions/72062851/problems-with-translations-with-vue-yup-and-i18n
const schema = object({
  email: string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
  name: string().required(() => t('error.required')),
  password: string()
    .required(() => t('error.required'))
    .min(8, () => t('error.minimum', { min: 8 })),
  confirmPassword: string()
    .required(() => t('error.required'))
    .oneOf([yupref('password')], () => t('error.passwordMissmatch')),
})

const formErrors = ref({})
provide('formErrors', formErrors)

function onInvalidSubmit({ values, errors, results }: any) {
  formErrors.value = { ...errors }
  console.error('invalid form data')
}
/**
 * @param values validated form data
 */
async function onSubmit(values: any) {
  formErrors.value = {}
  try {
    let response = await fetch(`${import.meta.env.VITE_API_URL}/api/register`, {
      method: 'POST',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ ...values }),
    })
    if (response.ok) {
      console.info('successful login')
      router.push({
        name: 'create-bracket',
      })
    } else {
      throw new Error('non-200 response for /api/report-result-for-bracket')
    }
  } catch (e) {
    console.error(e)
  }
}
</script>
