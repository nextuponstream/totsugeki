<template>
  <bracket-form @new-bracket="createBracket" />
</template>

<script setup lang="ts">
import BracketForm from '@/components/BracketForm.vue';
import { useRouter } from 'vue-router'

const router = useRouter()

function createBracket(name: string){
  localStorage.setItem('bracketName', name)
  router.push({
    name: 'registration',
  })
}
</script>
