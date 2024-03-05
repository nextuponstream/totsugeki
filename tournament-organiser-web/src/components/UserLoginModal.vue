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
import router, { RouteNames } from '@/router'
import { useI18n } from 'vue-i18n'
import { object, string, ref as yupref } from 'yup'
import { useUserStore } from '@/stores/user'
import { useBracketStore } from '@/stores/bracket'
import { useToastStore } from '@/stores/toast'

const { t } = useI18n({})
const userStore = useUserStore()
const bracketStore = useBracketStore()

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

function onInvalidSubmit({ values, errors, results }: any) {
  formErrors.value = { ...errors }
  console.error('invalid form data')
}

const toastStore = useToastStore()
/**
 * @param values validated form data
 */
async function onSubmit(values: any) {
  formErrors.value = {}
  let response = await userStore.login(values)
  switch (response) {
    case 200:
      emits('login')
      toastStore.success(t('login'))
      if (bracketStore.isSaved) {
        router.push({
          name: RouteNames.user.dashboard,
        })
      } else {
        router.push({
          name: RouteNames.bracket.guest,
        })
      }

      break
    case 404:
      console.warn('unknown email')
      setFieldError('email', t('error.unknownEmail'))
      break
    case 500:
      setFieldError('email', t('error.communication'))
      break
    default:
      console.error('unreachable')
      break
  }
}

const submitForm = handleSubmit((values: any) => {
  onSubmit(values)
}, onInvalidSubmit)
</script>
