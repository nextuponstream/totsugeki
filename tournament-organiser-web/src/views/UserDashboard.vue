<template>
  <div class="flex flex-col gap-8">
    <div>
      <div class="text-2xl">
        <h1>{{ $t('generic.profile') }}</h1>
      </div>
      <EditUser ref="editUser"></EditUser>
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
        <div>
          {{ $t('deleteModal.confirmWithMail', { email: infos.email }) }}
        </div>
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
import { onMounted, provide, reactive, ref, watch } from 'vue'
import { useForm } from 'vee-validate'
import { useI18n } from 'vue-i18n'
import { object, string } from 'yup'
import router from '@/router'
import EditUser from '@/components/EditUser.vue'

const emits = defineEmits(['logout'])
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
interface UserInfos {
  email: string
  name: string
}
const infos: UserInfos = reactive({ email: '', name: '' })

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
      let userInfos: { email: string; name: string } = await response.json()
      console.debug(JSON.stringify(userInfos))
      infos.email = userInfos.email
      infos.name = userInfos.name
    } else {
      throw new Error('non-200 response for /api/user')
    }
  } catch (e) {
    console.error(e)
  }
})
watch(infos, (first, second) => {
  console.debug('update edit form with', JSON.stringify(second))
  editUser.value?.setValues(infos)
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
  if (values.deleteEmail !== infos.email) {
    console.error('missmatch')
    console.debug(`${values.deleteEmail} !== ${infos.email}`)
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
