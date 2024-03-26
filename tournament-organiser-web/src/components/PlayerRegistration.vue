<template>
  <div class="grid grid-cols-1 gap-2">
    <div class="text-2xl">
      {{ t('playerRegistrationForm.title') }}
    </div>
    <form class="flex gap-2 align-baseline" @submit="submitForm">
      <FormInput
        v-model="name"
        v-bind="nameAttrs"
        name="name"
        :placeholder="$t('playerRegistrationForm.newPlayerPlaceholder')"
      />
      <SubmitBtn data-test-id="add-player" />
    </form>
  </div>
</template>
<script setup lang="ts">
import { useForm } from 'vee-validate'
import { ref, provide } from 'vue'
import { useI18n } from 'vue-i18n'
import { object, string } from 'yup'

const { t } = useI18n({})
const schema = object({
  name: string().required(() => t('error.required')),
})
const { resetForm, defineField, handleSubmit } = useForm({
  validationSchema: schema,
})
const [name, nameAttrs] = defineField('name')
const formErrors = ref({})
provide('formErrors', formErrors)
const emit = defineEmits(['newPlayer'])

function onInvalidSubmit({ values, errors, results }: any) {
  formErrors.value = { ...errors }
  console.error('invalid form data')
}
/**
 * @param values validated form data
 */
function onSubmit(values: any) {
  formErrors.value = {}
  console.info('TODO submit', JSON.stringify(values))
  emit('newPlayer', values.name)
  // https://vee-validate.logaretm.com/v4/guide/composition-api/handling-forms#handling-resets
  resetForm()
}

/**
 * Replace submit event with our handler such that we can use `resetForm`
 * NOTE https://dev.to/nickap/vee-validate-a-form-in-a-modal-useform-issue-when-used-in-a-modal-51ei
 */
const submitForm = handleSubmit((values: any) => {
  onSubmit(values)
}, onInvalidSubmit)
</script>
