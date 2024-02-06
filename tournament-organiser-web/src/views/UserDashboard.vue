<template>
  <div class="flex flex-col gap-8">
    <div>
      <div class="text-2xl">
        <h1>{{ $t('generic.profile') }}</h1>
      </div>
      <form
        class="flex flex-col max-w-xs gap-3"
        autocomplete="new-password"
        name="user-registration"
        @submit="submitForm"
      >
        <label>{{ $t('generic.email') }}</label>
        <FormInput
          v-model="email"
          name="email"
          type="email"
          v-bind="emailAttrs"
          :disabled="true"
        />
        <label>{{ $t('generic.username') }}</label>
        <FormInput
          v-model="name"
          name="name"
          type="text"
          v-bind="nameAttrs"
          :disabled="true"
        />
      </form>
    </div>
    <div class="flex flex-col max-w-xs gap-3">
      <div class="text-2xl text-red-700">
        <h1>{{ $t('user.dashboard.deleteAccount') }}</h1>
      </div>
      <DangerBtn @click="showDeleteModal">{{
        $t('user.dashboard.deleteMyAccount')
      }}</DangerBtn>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, provide, ref } from 'vue'
import { useForm } from 'vee-validate'
// import { useI18n } from 'vue-i18n'
import { object, string, ref as yupref } from 'yup'

// const { t } = useI18n({})
const schema = object({
  //   email: string()
  //     .email(() => t('error.invalidEmail'))
  //     .required(() => t('error.required')),
  //   name: string().required(() => t('error.required')),
})

const { resetForm, defineField, handleSubmit, setFieldError } = useForm({
  validationSchema: schema,
})

const [email, emailAttrs] = defineField('email')
const [name, nameAttrs] = defineField('name')

const formErrors = ref({})
provide('formErrors', formErrors)

// TODO remove after assessing if can get info from using cookie
onMounted(async () => {
  try {
    let response = await fetch(`${import.meta.env.VITE_API_URL}/api/user`, {
      method: 'GET',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
      },
    })
    if (response.ok) {
      let infos = await response.json()
      console.debug(JSON.stringify(infos))
      name.value = infos.name
      email.value = infos.email
    } else {
      throw new Error('non-200 response for /api/user')
    }
  } catch (e) {
    console.error(e)
  }
})

/**
 * @param values validated form data
 */
async function onSubmit(values: any) {
  // Do nothing, uncomment if you actually want to edit user settings
  //   formErrors.value = {}
  //   try {
  //     let response = await fetch(`${import.meta.env.VITE_API_URL}/api/register`, {
  //       method: 'POST',
  //       headers: {
  //         Accept: 'application/json',
  //         'Content-Type': 'application/json',
  //       },
  //       body: JSON.stringify({ ...values }),
  //     })
  //     if (response.ok) {
  //       console.info('successful login')
  //       router.push({
  //         name: 'createBracket',
  //       })
  //     } else if (response.status === 400) {
  //       let errorMessage: { message: string } = await response.json()
  //       if (errorMessage.message.includes('weak_password')) {
  //         setFieldError('password', t('error.weakPassword'))
  //       } else {
  //         throw new Error('non-200 response for /api/login')
  //       }
  //     } else {
  //       throw new Error('non-200 response for /api/report-result-for-bracket')
  //     }
  //   } catch (e) {
  //     console.error(e)
  //   }
}

function showDeleteModal() {
  // TODO
}

const submitForm = handleSubmit((values: any) => {
  onSubmit(values)
}, onInvalidSubmit)

function onInvalidSubmit({ values, errors, results }: any) {
  formErrors.value = { ...errors }
  console.error('invalid form data')
}
</script>
