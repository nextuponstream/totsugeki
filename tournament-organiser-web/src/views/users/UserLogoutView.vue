<template>
  <div></div>
</template>
<script setup lang="ts">
import { onMounted } from 'vue'
import { useUserStore } from '@/stores/user'
import router, { RouteNames } from '@/router'

const userStore = useUserStore()

onMounted(async () => {
  await userStore.logout()
  await router.push({ name: RouteNames.home })
})
</script>
