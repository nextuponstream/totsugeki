<template>
  <div class="fixed z-40 w-screen h-screen inset-0 bg-gray-900 bg-opacity-60" :class="isHidden"
    data-test-id="blurred-background-outside-modal" @click="hideModal" />
  <div
    class="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-auto bg-white rounded-md px-8 py-6 space-y-5 drop-shadow-lg"
    :class="isHidden" data-test-id="modal">
    <h1 class="text-2xl font-semibold">
      {{ title }}
    </h1>
    <slot></slot>
    <div>
      <CancelBtn @click="hideModal"></CancelBtn>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from "vue";

const emits = defineEmits(["update:modelValue", "hide"]);

const props = defineProps<{
  modelValue: boolean;
  title: string;
}>();

function hideModal() {
  emits('hide')
}

const isHidden = computed(() => {
  if (props.modelValue) {
    return null;
  } else {
    return "hidden";
  }
});
</script>
