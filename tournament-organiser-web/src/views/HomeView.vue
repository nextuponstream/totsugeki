<template>
  <div class="px-2 pt-2">
    <div
      v-if="submittedBracketName.length === 0"
      class="pb-2"
    >
      <bracket-form @new-bracket="registerBracket" />
    </div>
    <div v-else>
      <div class="text-xl">
        Bracket name: {{ submittedBracketName }}
      </div>
      <player-registration @new-player="addPlayer" />
      <player-seeder :players="playerList"/>
      <div>TODO reset bracket</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import BracketForm from '@/components/BracketForm.vue';
import PlayerSeeder from '@/components/PlayerSeeder.vue'
import PlayerRegistration from '@/components/PlayerRegistration.vue'
import { onMounted } from 'vue'
import { ref } from 'vue'
import type { Ref } from 'vue'

// https://github.com/SortableJS/vue.draggable.next/blob/master/example/components/simple.vue
onMounted(async () => {

});

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
