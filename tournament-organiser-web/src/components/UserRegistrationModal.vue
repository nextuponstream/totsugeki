<template>
  <BaseModal
    v-model="showModal"
    :title="$t('loginModal.title')"
    @hide="hideModal"
  >
    <Form class="flex flex-col gap-1">
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
      <div class="mt-2">
        {{ $t('loginModal.text1') }}
        <BaseLink
          href="/register"
          class=""
        >
          {{ $t('loginModal.text2') }}
        </BaseLink>
      </div>
      <SubmitBtn class="self-end" />
    </Form>
  </BaseModal>
</template>
<script setup lang="ts">
// NOTE submit button on the left for forms, right for modals (recommendation https://ux.stackexchange.com/a/13539)
import { Form } from 'vee-validate';
import { ref, watchEffect } from 'vue';
import BaseModal from './ui/BaseModal.vue';
const props = defineProps<{
    modelValue: boolean;
}>();
const showModal = ref(false);
const emits = defineEmits(["update:modelValue"]);

watchEffect(() => {
    showModal.value = props.modelValue
})

function hideModal() {
    emits('update:modelValue', false)
}

// TODO submit login to API

</script>