<template>
  <bracket-form @new-bracket="createBracket" />
</template>

<script setup lang="ts">
import BracketForm from '@/components/BracketForm.vue'
import { useRouter } from 'vue-router'

const router = useRouter()

function createBracket(name: string) {
  router.push({
    name: 'bracketRegistration',
  })
}
</script>
