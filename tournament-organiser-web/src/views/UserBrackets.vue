<template>
  <div>{{ JSON.stringify(bracketStore.bracketList) }}</div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useUserStore } from '@/stores/user'
import { useBracketStore } from '@/stores/bracket'

const userStore = useUserStore()
const bracketStore = useBracketStore()

onMounted(async () => {
  await bracketStore.getBracketsFrom(userStore.id!)
})
</script>
<style scoped></style>
