<template>
  <BaseModal
    v-model="showModal"
    :title="$t('loginModal.title')"
    @hide="hideModal"
  >
    <form
      class="flex flex-col gap-2"
      :validation-schema="schema"
      autocomplete="on"
      name="login"
      @submit="submitForm"
    >
      <label>{{ $t('generic.email') }}</label>
      <FormInput name="email" type="email" />
      <label>{{ $t('generic.password') }}</label>
      <FormInput name="password" type="password" />
      <div class="mt-2">
        {{ $t('loginModal.text1') }}
        <BaseLink href="/register" class="">
          {{ $t('loginModal.text2') }}
        </BaseLink>
      </div>
      <SubmitBtn class="self-end" />
    </form>
  </BaseModal>
</template>
<script setup lang="ts">
// NOTE submit button on the left for forms, right for modals (recommendation https://ux.stackexchange.com/a/13539)
import { useForm } from 'vee-validate'
import { ref, watchEffect, provide } from 'vue'
import BaseModal from './ui/BaseModal.vue'
import router from '@/router'
import { useI18n } from 'vue-i18n'
import { object, string, ref as yupref } from 'yup'

const { t } = useI18n({})

const props = defineProps<{
  modelValue: boolean
}>()
const showModal = ref(false)
const schema = object({
  email: string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
  password: string()
    .required(() => t('error.required'))
    .min(8, () => t('error.minimum', { min: 8 })),
})
const emits = defineEmits(['update:modelValue', 'login'])

const formErrors = ref({})
provide('formErrors', formErrors)

watchEffect(() => {
  showModal.value = props.modelValue
})

function hideModal() {
  emits('update:modelValue', false)
}

const { resetForm, defineField, handleSubmit, setFieldError } = useForm({
  validationSchema: schema,
})

const [email, emailAttrs] = defineField('email')
const [password, passwordAttrs] = defineField('password')

// TODO submit login to API
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
    let response = await fetch(`${import.meta.env.VITE_API_URL}/api/login`, {
      method: 'POST',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
        Authorization: 'Basic ' + btoa(`${values.email}:${values.password}`),
      },
      credentials: 'same-origin',
      body: JSON.stringify({ ...values }),
    })
    if (response.ok) {
      console.info('successful login')
      if (response.body) {
        let body = await response.json()
        // store user ID and prefer logged in view from now on
        localStorage.setItem('user_id', body.user_id)
        router.push({
          name: 'userDashboard',
        })
        emits('login')
      } else {
        throw new Error('expected json response')
      }
    } else if (response.status === 404) {
      console.warn('unknown email')
      setFieldError('email', t('error.unknownEmail'))
    } else {
      throw new Error('non-200 response for /api/login')
    }
  } catch (e) {
    console.error(e)
  }
}

const submitForm = handleSubmit((values: any) => {
  onSubmit(values)
}, onInvalidSubmit)
</script>
