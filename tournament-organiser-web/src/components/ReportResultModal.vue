<template>
  <div
    class="fixed z-40 w-screen h-screen inset-0 bg-gray-900 bg-opacity-60"
    :class="isHidden"
    @click="hideModal"
  />
  <div
    class="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-auto bg-white rounded-md px-8 py-6 space-y-5 drop-shadow-lg"
    :class="isHidden"
  >
    <h1 class="text-2xl font-semibold">
      Match results
    </h1>
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
      <OtherBtn @click="setScore(2, 0)">
        2 - 0
      </OtherBtn>
      <OtherBtn @click="setScore(2, 1)">
        2 - 1
      </OtherBtn>
      <OtherBtn @click="setScore(1, 2)">
        1 - 2
      </OtherBtn>
      <OtherBtn @click="setScore(0, 2)">
        0 - 2
      </OtherBtn>
    </div>
    <div class="grid sm:grid-cols-6 gap-2">
      <OtherBtn @click="setScore(3, 0)">
        3 - 0
      </OtherBtn>
      <OtherBtn @click="setScore(3, 1)">
        3 - 1
      </OtherBtn>
      <OtherBtn @click="setScore(3, 2)">
        3 - 2
      </OtherBtn>
      <OtherBtn @click="setScore(2, 3)">
        2 - 3
      </OtherBtn>
      <OtherBtn @click="setScore(1, 3)">
        1 - 3
      </OtherBtn>
      <OtherBtn @click="setScore(0, 3)">
        0 - 3
      </OtherBtn>
    </div>
    <div class="grid sm:grid-cols-3">
      <div>{{ players ? players[0].name : "" }}</div>
      <div class="place-self-center">
        <div>{{ scoreP1 }} - {{ scoreP2 }}</div>
      </div>
      <div class="flex flex-row-reverse">
        {{ players ? players[1].name : "" }}
      </div>
    </div>
    <div class="flex gap-1">
      <SubmitBtn
        :disabled="isSubmitDisabled"
        @click="submit"
      />
      <CancelBtn @click="hideModal" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, computed, onUpdated } from "vue";

const props = defineProps<{
  matchId: string | null;
  players: { name: string; id: string }[] | null;
  modelValue: boolean;
}>();

const hide = ref(false);
const scoreP1 = ref(0);
const scoreP2 = ref(0);
const emits = defineEmits(["update:modelValue", "newResult"]);

onUpdated(() => {
  if (!props.modelValue) {
    scoreP1.value = 0;
    scoreP2.value = 0;
  }
});

function hideModal() {
  hide.value = true;
  emits("update:modelValue", false);
}

function setScore(p1: number, p2: number) {
  scoreP1.value = p1;
  scoreP2.value = p2;
}

function submit() {
  emits("newResult", props.players, scoreP1.value, scoreP2.value);
  hideModal();
}

const isHidden = computed(() => {
  if (props.modelValue) {
    return null;
  } else {
    return "hidden";
  }
});
const isSubmitDisabled = computed(() => {
  return scoreP1.value === 0 && scoreP2.value === 0;
});
</script>
