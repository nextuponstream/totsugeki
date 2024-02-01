<template>
  <BaseModal
    v-model="showModal"
    :title="$t('loginModal.title')"
    @hide="hideModal"
  >
    <Form
      class="flex flex-col gap-2"
      :on-invalid-submit="onInvalidSubmit"
      :on-submit="onSubmit"
      :validation-schema="schema"
      autocomplete
      name="login"
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
    </Form>
  </BaseModal>
</template>
<script setup lang="ts">
// NOTE submit button on the left for forms, right for modals (recommendation https://ux.stackexchange.com/a/13539)
import { Form } from 'vee-validate'
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
const emits = defineEmits(['update:modelValue'])

const formErrors = ref({})
provide('formErrors', formErrors)

watchEffect(() => {
  showModal.value = props.modelValue
})

function hideModal() {
  emits('update:modelValue', false)
}

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
      // TODO redirect to logged in view
      // router.push({
      //   name: 'create-bracket',
      // })
    } else {
      throw new Error('non-200 response for /api/login')
    }
  } catch (e) {
    console.error(e)
  }

  // TODO remove after assessing if can get info from using cookie
  try {
    let response = await fetch(`${import.meta.env.VITE_API_URL}/api/user`, {
      method: 'GET',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
    })
    if (response.ok) {
      console.info('get infos?')
    } else {
      throw new Error('non-200 response for /api/user')
    }
  } catch (e) {
    console.error(e)
  }
}
</script>
