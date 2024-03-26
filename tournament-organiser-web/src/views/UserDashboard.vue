<template>
  <div class="flex flex-col gap-8">
    <div>
      <div class="text-2xl">
        <h1>{{ t('generic.profile') }}</h1>
      </div>
      <EditUser ref="editUser"></EditUser>
    </div>
    <div class="flex flex-col max-w-xs gap-3">
      <div class="text-2xl text-red-700">
        <h1>{{ t('user.dashboard.deleteAccount') }}</h1>
      </div>
      <DangerBtn @click="showDeleteModal">{{
        t('user.dashboard.deleteMyAccount')
      }}</DangerBtn>
    </div>
    <BaseModal
      v-model="showModal"
      :title="t('deleteModal.title')"
      @hide="hideModal"
    >
      <form
        class="flex flex-col gap-2"
        :validation-schema="deleteSchema"
        autocomplete="off"
        name="login"
        @submit="onDeleteAccountFormSubmit"
      >
        <div>
          {{
            t('deleteModal.confirmWithMail', { email: userStore.infos.email })
          }}
        </div>
        <label>{{ t('generic.email') }}</label>
        <FormInput
          name="deleteEmail"
          type="email"
          autocomplete="off"
          v-bind="deleteEmailAttrs"
        />
        <DangerBtn class="self-end">{{
          t('user.dashboard.deleteAccount')
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
import EditUser from '@/components/EditUser.vue'
import { useUserStore } from '@/stores/user'

const userStore = useUserStore()
const { t } = useI18n({})
const editUser = ref<InstanceType<typeof EditUser> | null>(null)
const deleteSchema = object({
  deleteEmail: string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
})
const deleteForm = useForm({
  validationSchema: deleteSchema,
})

// deleteModal
const [deleteEmail, deleteEmailAttrs] = deleteForm.defineField('deleteEmail')

const deleteFormErrors = ref({})
provide('formErrors', deleteFormErrors)

const showModal = ref(false)

onMounted(async () => {
  await userStore.getUser()
  editUser.value?.setValues(userStore.infos)
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
  if (values.deleteEmail !== userStore.infos.email) {
    console.error('mismatch')
    console.debug(`${values.deleteEmail} !== ${userStore.infos.email}`)
    deleteForm.setFieldError('deleteEmail', t('deleteModal.matchError'))
    return
  }

  // all ok
  let deleted = await userStore.deleteAccount()
  if (deleted) {
    await router.push({ name: 'createBracket' })
  }
}
</script>
