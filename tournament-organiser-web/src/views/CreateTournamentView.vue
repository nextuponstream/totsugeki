<template>
  <tournament-form @new-tournament="createTournament" />
</template>

<script setup lang="ts">
import TournamentForm from '@/components/TournamentForm.vue'
import { useRouter } from 'vue-router'

const router = useRouter()

function createTournament() {
  router.push({
    name: 'tournamentCreate',
  })
}
</script>
