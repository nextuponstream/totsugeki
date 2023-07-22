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
      <div class="flex flex-row gap-7">
        <div class="pt-6">
          <player-registration @new-player="addPlayer" />
        </div>

        <div class="pt-6">
          <player-seeder :players="playerList" />
        </div>
      </div>

      <div class="group mt-5">
        <submit-btn
          :disabled="playerList.length < 3"
          @click="openConfirmModal"
        >
          {{ t('home.startBracket') }}
        </submit-btn>
        <base-tooltip
          v-if="playerList.length < 3"
          class="ml-3"
        >
          3 players minimum
        </base-tooltip>
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

function openConfirmModal(){
  // TODO
}
</script>
