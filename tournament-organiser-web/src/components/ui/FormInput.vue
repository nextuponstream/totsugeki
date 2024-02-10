<template>
  <div class="flex flex-col gap-1">
    <input
      :name="name"
      :type="type"
      :placeholder="placeholder"
      :value="value"
      :class="inputClasses"
      class="p-2 rounded-md border-solid"
      :disabled="disabled"
      :autocomplete="autocomplete"
      v-on="handlers"
    />
    <ErrorMessage :name="name" class="inputErrorMessage" />
  </div>
</template>
<script setup lang="ts">
import { computed, inject, toRef, watchEffect, type Ref } from 'vue'
import type { FieldContext } from 'vee-validate'
import { useField, ErrorMessage } from 'vee-validate'

type InteractionEventGetter = (ctx: FieldContext) => string[]
const props = defineProps({
  name: {
    type: String,
    default: '',
  },
  mode: {
    type: String,
    default: 'passive',
  },
  type: {
    type: String,
    default: 'text',
  },
  placeholder: {
    type: String,
    default: '',
    required: false,
  },
  disabled: {
    type: Boolean,
    default: false,
    required: false,
  },
  autocomplete: {
    type: String,
    default: 'on',
    required: false,
  },
})
/**
 * All errors from parent form. Used to update visual state of the input
 */
const formErrors = inject<Ref>('formErrors')

watchEffect(() => {
  // NOTE: uncomment to watch changes in reactive injected keys here
  // if (formErrors) {
  // }
})

/**
 * Adds (in-)valid classes to the html input
 * NOTE: tailwind invalid prefix only apply to basic html input validation and not yup validation
 */
const validClasses = computed(() => {
  const propertyName = props.name
  if (formErrors) {
    const errors = formErrors?.value
    if (propertyName in errors) {
      console.debug(`${propertyName} has error ${errors[propertyName]}`)
      return 'border border-red-500'
    }
  }

  // NOTE outline-none is the blue border thingy when focusing
  return 'border border-gray-300 focus:ring focus:ring-indigo-300 outline-none focus:ring-opacity-50'
})

const disabledClasses = computed(() => {
  if (props.disabled) {
    return 'text-gray-500'
  } else {
    return ''
  }
})

const inputClasses = computed(() => {
  return `${disabledClasses.value} ${validClasses.value}`
})
// use `toRef` to create reactive references to `name` prop which is passed to `useField`
// this is important because vee-validte needs to know if the field name changes
// https://vee-validate.logaretm.com/v4/guide/composition-api/caveats
const { meta, value, errorMessage, handleChange, handleBlur } = useField(
  toRef(props, 'name')
)
/**
 * Only validate when submitting the form. This should be the default to prevent fatigue
 */
const passive: InteractionEventGetter = () => []
/**
 * Show error after leaving the input
 * @param param0
 */
const lazy: InteractionEventGetter = ({ meta, errorMessage }) => {
  return ['change']
}
/**
 * Show error while filling the input (and maybe smth?)
 */
const aggressive: InteractionEventGetter = () => ['input']
/**
 * Starts lazy, then turns aggressive after leaving the input in an invalid state,
 * then turns back to lazy after it becomes valid
 * From: https://dev.to/vponamariov/validate-it-ultimate-guide-41d1#trigger-eager
 * @param param0
 */
const eager: InteractionEventGetter = ({ meta, errorMessage }) => {
  if (errorMessage.value) {
    return ['input']
  }

  return ['change']
}
// Validation modes official code example https://vee-validate.logaretm.com/v4/examples/dynamic-validation-triggers/
// TODO remove any type from example code
const modes: any = {
  passive,
  lazy,
  aggressive,
  eager,
}

// generates the listeners
const handlers = computed(() => {
  // TODO remove any type from example code
  const on: any = {
    blur: handleBlur,
    // default input event to sync the value
    // the `false` here prevents validation
    // TODO remove any type from example code
    input: [(e: any) => handleChange(e, false)],
  }

  // Get list of validation events based on the current mode
  const triggers = modes[props.mode]({
    errorMessage,
    meta,
  })

  // add them to the "on" handlers object
  triggers.forEach((t: any) => {
    if (Array.isArray(on[t])) {
      on[t].push(handleChange)
    } else {
      on[t] = handleChange
    }
  })

  return on
})
</script>
<style scoped>
.inputErrorMessage {
  color: red;
  font-size: small;
}
</style>
