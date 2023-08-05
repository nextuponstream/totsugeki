<template>
  <div
    class="fixed z-40 w-screen h-screen inset-0 bg-gray-900 bg-opacity-60"
    :class="isHidden"
    @click="hideModal"
  />
  <div
    class="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 bg-white rounded-md px-8 py-6 space-y-5 drop-shadow-lg"
    :class="isHidden"
  >
    <h1 class="text-2xl font-semibold">
      Match results
    </h1>
    <button>2-0</button><button>2-1</button><button>1-2</button><button>0-2</button>
    <div class="flex flex-row">
      <div>{{ players ? players[0] : '' }}</div>
      <div>{{ players ? players[1] : '' }}</div>
    </div>
    <SubmitBtn />
    <button @click="hideModal">
      Close
    </button>
  </div>
</template>
<script setup lang="ts">

import {ref, computed} from 'vue'
import SubmitBtn from './ui/SubmitBtn.vue';

const props = defineProps<{
    matchId: string | null,
    players: string[] | null,
    modelValue: boolean,
}>()

const hide = ref(false)
const emits = defineEmits(['update:modelValue'])

function hideModal() {
    hide.value = true
    emits('update:modelValue', false)
}

const isHidden = computed(() => {
    if (props.modelValue) {
        return null
    } else {
        return 'hidden'
    }
})

</script>