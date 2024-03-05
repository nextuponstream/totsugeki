<template>
  <Teleport to="body">
    <div class="toastZone grid grid-flow-row gap-4">
      <div v-for="toast in toasts" :key="toast.id">
        <BaseToast
          :title="toast.title"
          :text="toast.content"
          :type="toast.type"
        ></BaseToast>
      </div>
    </div>
  </Teleport>
</template>
<script setup lang="ts">
import { useToastStore, type ToastStatus } from '@/stores/toast'
import { storeToRefs } from 'pinia'
import BaseToast from './ui/BaseToast.vue'

const toastStore = useToastStore()

const { toasts } = storeToRefs(toastStore)
</script>
<style scoped>
.toastZone {
  position: fixed;
  top: 3%;
  right: 10%;
}
</style>
