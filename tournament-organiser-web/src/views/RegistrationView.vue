<template>
  <Form
    class="flex flex-col max-w-xs gap-3"
    :on-submit="onSubmit"
    :on-invalid-submit="onInvalidSubmit"
    :validation-schema="schema"
  >
    <label>{{ $t('generic.email') }}</label>
    <FormInput
      name="email"
      type="email"
    />
    <label>{{ $t('generic.password') }}</label>
    <FormInput
      name="password"
      type="password"
    />
    <label>{{ $t('generic.confirmPassword') }}</label>
    <FormInput
      name="confirmPassword"
      type="password"
    />
    <SubmitBtn>{{ $t('generic.register') }}</SubmitBtn>
  </Form>
</template>
<script setup lang="ts">
// TODO when an error is displayed, locale of errors does not update automatically on locale change
// tried looking online but it's a bad interaction between vee-validate and i18n I guess.
// However, it's not a big concern
import { Form } from 'vee-validate';
import { ref, provide } from 'vue';
import { object, string, ref as yupref } from 'yup';
import { useI18n } from 'vue-i18n';
const { t } = useI18n({});
// NOTE: how to use i18n with yup https://stackoverflow.com/questions/72062851/problems-with-translations-with-vue-yup-and-i18n
const schema = object({
  email: string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
  password: string().required(() => t('error.required')).min(8, () => t('error.minimum', { min: 8 })),
  confirmPassword: string().required(() => t('error.required'))
    .oneOf([yupref('password')], () => t('error.passwordMissmatch')),
});

const formErrors = ref({})
provide('formErrors', formErrors)

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
}

</script>
