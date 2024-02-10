<template>
  <div class="flex flex-col gap-8">
    <div>
      <div class="text-2xl">
        <h1>{{ $t('generic.profile') }}</h1>
      </div>
      <form
        class="flex flex-col max-w-xs gap-3"
        autocomplete="new-password"
        name="user-edit"
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
    <BaseModal
      v-model="showModal"
      :title="$t('deleteModal.title')"
      @hide="hideModal"
    >
      <form
        class="flex flex-col gap-2"
        :validation-schema="deleteSchema"
        autocomplete="off"
        name="login"
        @submit="onDeleteAccountFormSubmit"
      >
        <div>{{ $t('deleteModal.confirmWithMail', { email }) }}</div>
        <label>{{ $t('generic.email') }}</label>
        <FormInput
          name="deleteEmail"
          type="email"
          autocomplete="off"
          v-bind="deleteEmailAttrs"
        />
        <DangerBtn class="self-end">{{
          $t('user.dashboard.deleteAccount')
        }}</DangerBtn>
      </form>
    </BaseModal>
  </div>
</template>
<script setup lang="ts">
import { onMounted, provide, ref } from 'vue'
import { useForm } from 'vee-validate'
import { useI18n } from 'vue-i18n'
import { object, string } from 'yup'
import router from '@/router'

const emits = defineEmits(['logout'])

const { t } = useI18n({})

const editForm = useForm({})
const deleteSchema = object({
  deleteEmail: string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
})
const deleteForm = useForm({
  validationSchema: deleteSchema,
})

// User info form
const [email, emailAttrs] = editForm.defineField('email')
const [name, nameAttrs] = editForm.defineField('name')

// deleteModal
const [deleteEmail, deleteEmailAttrs] = deleteForm.defineField('deleteEmail')

const deleteFormErrors = ref({})
provide('formErrors', deleteFormErrors)

const showModal = ref(false)

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

function showDeleteModal() {
  showModal.value = true
}

function hideModal() {
  showModal.value = false
}

const onDeleteAccountFormSubmit = deleteForm.handleSubmit(
  deleteAccountFormSubmit
)

async function deleteAccountFormSubmit(values: any) {
  // special validation
  if (values.deleteEmail !== email.value) {
    console.error('missmatch')
    console.debug(`${values.deleteEmail} !== ${email.value}`)
    deleteForm.setFieldError('deleteEmail', t('deleteModal.matchError'))
    return
  }

  // all ok
  try {
    let response = await fetch(`${import.meta.env.VITE_API_URL}/api/user`, {
      method: 'DELETE',
    })
    if (response.ok) {
      console.info('successful account deletion')
      localStorage.removeItem('user_id')
      emits('logout')
      router.push({
        name: 'createBracket',
      })
    }
  } catch (e) {
    console.error(e)
  }
}
</script>
