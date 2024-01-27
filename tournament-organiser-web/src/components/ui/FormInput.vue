<template>
  <input :type="type" :value="value" v-on="handlers" :class="invalidClasses"
    class="p-2 rounded-md border-gray-300 focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50" />
  <ErrorMessage :name="name" style="color:red;" />
</template>
<script setup lang="ts">
import { computed, inject, ref, toRef, toRefs, watch, watchEffect, type Ref } from 'vue';
import type { FieldContext } from 'vee-validate';
import { useField, ErrorMessage } from 'vee-validate';

type InteractionEventGetter = (ctx: FieldContext) => string[];
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
    default: 'text'
  },
});
const submittedOnce = inject<Ref<boolean>>('submittedOnce')

watchEffect(() => {
  if (submittedOnce) {
    console.debug('new provided submittedOnce', submittedOnce.value)
  }
})

/**
 * Sets invalid class only after submitting
 * FIXME red border not visible when empty
 */
const invalidClasses = computed(() => {
  // console.log('change')
  return submittedOnce?.value ? 'invalid:border invalid:border-red-500' : ''
})
// use `toRef` to create reactive references to `name` prop which is passed to `useField`
// this is important because vee-validte needs to know if the field name changes
// https://vee-validate.logaretm.com/v4/guide/composition-api/caveats
const { meta, value, errorMessage, handleChange, handleBlur } = useField(
  // toRef(props.name)
  // FIXME remove type error from code example
  toRef(props, 'name'),
);
const passive: InteractionEventGetter = () => [];

// const lazy: InteractionEventGetter = ({ meta, errorMessage }) => {
//   return ['change'];
// };

// const aggressive: InteractionEventGetter = () => ['input'];

// const eager: InteractionEventGetter = ({ meta, errorMessage }) => {
//   if (errorMessage.value) {
//     return ['input'];
//   }

//   return ['change'];
// };
// FIXME remove any type from example code
const modes: any = {
  passive,
  // lazy,
  // aggressive,
  // eager,
};

// // generates the listeners
const handlers = computed(() => {
  // FIXME remove any type from example code
  const on: any = {
    blur: handleBlur,
    // default input event to sync the value
    // the `false` here prevents validation
    // FIXME remove any type from example code
    input: [(e: any) => handleChange(e, false)],
  };

  // Get list of validation events based on the current mode
  const triggers = modes[props.mode]({
    errorMessage,
    meta,
  });

  // add them to the "on" handlers object
  triggers.forEach((t: any) => {
    if (Array.isArray(on[t])) {
      on[t].push(handleChange);
    } else {
      on[t] = handleChange;
    }
  });

  return on;
});
</script>
<style scoped></style>
