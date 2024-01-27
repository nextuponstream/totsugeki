<template>
  <div class="grid grid-cols-1 gap-2">
    <div class="text-2xl">
      {{ t("bracketForm.title") }}
    </div>
    <Form
      :on-submit="onSubmit"
      :on-invalid-submit="onInvalidSubmit"
      :validation-schema="schema"
      class="flex flex-col gap-2 max-w-xs"
    >
      <div>
        <label>{{ t("bracketForm.nameLabel") }}</label>
        <FormInput
          name="bracket"
          :placeholder="t('bracketForm.namePlaceholder')"
        />
      </div>
      <SubmitBtn />
    </Form>
  </div>
</template>
<script setup lang="ts">
import { Form } from 'vee-validate';
import { ref, provide } from 'vue';
import * as yup from 'yup';
import { useI18n } from 'vue-i18n';
const { t } = useI18n({});
// NOTE: how to use i18n with yup https://stackoverflow.com/questions/72062851/problems-with-translations-with-vue-yup-and-i18n
const schema = yup.object({
  bracket: yup.string().required(() => t('error.required')),
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
  emit("newBracket", values.bracket);
}

const emit = defineEmits(["newBracket"]);

</script>
