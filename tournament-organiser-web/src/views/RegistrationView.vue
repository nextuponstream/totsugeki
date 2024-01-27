<template>
  <Form class="flex flex-col max-w-xs gap-3" @submit="onSubmit" :validation-schema="schema">
    <label>{{ $t('generic.email') }}</label>
    <!-- :submittedOnce="submittedOnce" -->
    <FormInput name="email" type="email"></FormInput>
    <SubmitBtn @click="onSubmit"></SubmitBtn>
  </Form>
</template>
<script setup lang="ts">
// passive to aggressive validation has been removed??? https://github.com/logaretm/vee-validate/issues/379
// FIXME when an error is displayed, locale of errors does not update automatically
// tried looking online but it's a bad interaction between vee-validate and i18n I guess
import { Form } from 'vee-validate';
import { ref, provide } from 'vue';
import * as yup from 'yup';
import { useI18n } from 'vue-i18n';
const { t } = useI18n({});
const schema = yup.object({
  email: yup.string()
    .email(() => t('error.invalidEmail'))
    .required(() => t('error.required')),
});
const submittedOnce = ref(false)

function onSubmit(values: any) {
  submittedOnce.value = true
  // FIXME only observ
  console.debug('hello')
  console.debug(JSON.stringify(values, null, 2));
}

provide('submittedOnce', submittedOnce)

// TODO USE THIS FOR WITH PASSIVE VALIDATION https://vee-validate.logaretm.com/v4/examples/dynamic-validation-triggers/

// TODO apply confirmed from https://vee-validate.logaretm.com/v4/guide/global-validators

// NOTE people dissatisfied with vee-validate 4 https://github.com/logaretm/vee-validate/issues/3088
</script>
