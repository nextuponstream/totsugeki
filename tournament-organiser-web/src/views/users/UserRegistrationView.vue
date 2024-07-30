<template>
  <form
    class="flex flex-col max-w-xs gap-3"
    autocomplete="new-password"
    name="user-registration"
    @submit="submitForm"
  >
    <label>{{ $t('generic.email') }}</label>
    <FormInput v-model="email" name="email" type="email" v-bind="emailAttrs" />
    <label>{{ $t('generic.username') }}</label>
    <FormInput v-model="name" name="name" type="text" v-bind="nameAttrs" />
    <label>{{ $t('generic.password') }}</label>
    <FormInput
      v-model="password"
      name="password"
      type="password"
      v-bind="passwordAttrs"
    />
    <label>{{ $t('generic.confirmPassword') }}</label>
    <FormInput
      v-model="confirmPassword"
      name="confirmPassword"
      type="password"
      v-bind="confirmPasswordAttrs"
    />
    <SubmitBtn>{{ $t('generic.register') }}</SubmitBtn>
  </form>
</template>
<script setup lang="ts">
// TODO when an error is displayed, locale of errors does not update automatically on locale change
// tried looking online but it's a bad interaction between vee-validate and i18n I guess.
// However, it's not a big concern
import { useForm } from 'vee-validate'
import { ref, provide } from 'vue'
import { object, string, ref as yupref } from 'yup'
import { useI18n } from 'vue-i18n'
import router, { RouteNames } from '@/router'
import { useUserStore } from '@/stores/user'

const userStore = useUserStore()

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

const { defineField, handleSubmit, setFieldError } = useForm({
  validationSchema: schema,
})

const [email, emailAttrs] = defineField('email')
const [name, nameAttrs] = defineField('name')
const [password, passwordAttrs] = defineField('password')
const [confirmPassword, confirmPasswordAttrs] = defineField('confirmPassword')

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
  const response = await userStore.registration(values)
  switch (response) {
    case 200:
      await router.push({ name: RouteNames.home })
      break
    case 500:
      break
    default:
      if (response.includes('weak_password')) {
        setFieldError('password', t('error.weakPassword'))
      } else {
        throw new Error('non-200 response for /api/login')
      }
  }
}

const submitForm = handleSubmit((values: any) => {
  onSubmit(values)
}, onInvalidSubmit)
</script>
