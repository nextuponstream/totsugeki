<template>
    <BaseModal :title="$t('loginModal.title')" v-model="modelValue" v-on:hide="hideModal">
        <div class="flex flex-col gap-2">
            <label>{{ $t('generic.email') }}</label>
            <!-- <FormInput v-model="email"></FormInput> -->
            <label>{{ $t('generic.password') }}</label>
            <!-- <FormInput v-model="password"></FormInput> -->
            <div class="mt-2">
                {{ $t('loginModal.text1') }}
                <BaseLink href="/register" class="">{{ $t('loginModal.text2') }}</BaseLink>
            </div>
        </div>
    </BaseModal>
</template>
<script setup lang="ts">
import { ref } from 'vue';
import BaseModal from './ui/BaseModal.vue';
const props = defineProps<{
    modelValue: boolean;
}>();
const emits = defineEmits(["update:modelValue"]);

// const email = ref('')
// const password = ref('')

function hideModal() {
    emits('update:modelValue', false)
}

</script>