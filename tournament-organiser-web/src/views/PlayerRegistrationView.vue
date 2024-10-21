<template>
  <form @submit="submitForm">
    <div class="sm:grid sm:gap-5">
      <div class="sm:grid sm:grid-cols-5">
        <form-input
          v-model="bracketName"
          v-bind="bracketNameAttrs"
          name="bracketName"
          :placeholder="$t('bracketForm.namePlaceholder')"
        ></form-input>
      </div>

      <div class="flex">
        <submit-btn data-test-id="start-bracket">
          {{ $t('registration.startBracket') }}
        </submit-btn>
      </div>
    </div>
  </form>
  <div class="pt-4 sm:grid sm:grid-cols-2 sm:gap-5">
    <div class="pb-5">
      <player-registration @new-player="bracketStore.addPlayerInForm" />
      <div class="group mt-5 grid grid-cols-1 place-items-center">
        <div></div>
      </div>
    </div>

    <div>
      <player-seeder />
    </div>
  </div>
</template>

<script setup lang="ts">
import PlayerSeeder from '@/components/PlayerSeeder.vue'
import PlayerRegistration from '@/components/PlayerRegistration.vue'
import { onMounted, provide, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useBracketStore } from '@/stores/bracket'
import { useUserStore } from '@/stores/user'
import FormInput from '@/components/ui/FormInput.vue'
import { object, string } from 'yup'
import { useForm, Form } from 'vee-validate'

// NOTE: while not very important, I find it disturbing that the "save bracket"
// button is not put at the bottom of the page. But then, I don't want a form
// within a form (placing that button inside player-registration component) and
// I don't want bracket form input at the bottom of the page

const { t } = useI18n({})
const router = useRouter()

const bracketStore = useBracketStore()
const userStore = useUserStore()

const schema = object({
  bracketName: string().required(() => t('error.required')),
})
const { defineField, handleSubmit, setFieldValue } = useForm({
  validationSchema: schema,
})

const [bracketName, bracketNameAttrs] = defineField('name')

async function createBracketFromPlayers() {
  let loggedIn: boolean = userStore.id !== null
  try {
    await bracketStore.createBracket(loggedIn)
    if (loggedIn) {
      await router.push({
        name: 'bracket',
        params: { bracketId: bracketStore.id },
      })
    } else {
      await router.push({ name: 'bracket-guest' })
    }
  } catch (e) {
    console.error(e)
  }
}

const formErrors = ref({})
// Intuitively, you should not provide formErrors twice (both here and in player
// registration). But it seems to work well enough.
provide('formErrors', formErrors)

// FIXME empty bracket name does not create 401

async function onSubmit(values: { bracketName: string }) {
  bracketStore.formCreate.bracket_name = values.bracketName
  await createBracketFromPlayers()
}

function onInvalidSubmit({ values, errors, results }: any) {
  formErrors.value = { ...errors }
  console.error('invalid form data')
}

const submitForm = handleSubmit((values: any) => {
  onSubmit(values)
}, onInvalidSubmit)

onMounted(() => {
  setFieldValue('bracketName', bracketStore.formCreate.bracket_name)
})
</script>
