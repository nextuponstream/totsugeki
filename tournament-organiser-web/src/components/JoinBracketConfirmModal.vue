<template>
  <BaseModal :title="$t('joinBracket.title')" :model-value="model" @hide="hide">
    <other-btn @click="emits('confirmed')"
      >{{ $t('joinBracketConfirmModal.joinButton') }}
    </other-btn>
  </BaseModal>
</template>
<script setup lang="ts">
import BaseModal from '@/components/ui/modals/BaseModal.vue'
import OtherBtn from '@/components/ui/buttons/OtherBtn.vue'

const model = defineModel({ default: () => false })
const emits = defineEmits(['confirmed'])

function hide() {
  model.value = false
}
</script>
<style scoped></style>
