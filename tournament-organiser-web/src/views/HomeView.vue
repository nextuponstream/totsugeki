<template>
  <div class="px-2">
    <div
      v-if="submittedBracketName.length === 0"
      class="pb-2"
    >
      <bracket-form @new-bracket="registerBracket" />
    </div>
    <div v-else>
      <div class="text-xl">
        {{ t('home.bracketNameLabel') }}: {{ submittedBracketName }}
      </div>
      <div class="pt-6">
        <player-registration @new-player="addPlayer" />
      </div>
      <div class="pt-6">
        <player-seeder :players="playerList" />
      </div>
      <div class="pt-6">
        <submit-btn></submit-btn>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import BracketForm from '@/components/BracketForm.vue';
import PlayerSeeder from '@/components/PlayerSeeder.vue'
import PlayerRegistration from '@/components/PlayerRegistration.vue'
import { ref } from 'vue'
import type { Ref } from 'vue'
import { useI18n } from 'vue-i18n';
import SubmitBtn from '@/components/SubmitBtn.vue';

const {t} = useI18n({})

const playerList : Ref<{name: string}[]>= ref([])

const dragging = ref(false)
const enabled = ref(true)
const submittedBracketName = ref('')

function registerBracket(name: string): void {
  submittedBracketName.value = name
}

function addPlayer(name: string): void {
  playerList.value.push({name: name})
}
</script>
