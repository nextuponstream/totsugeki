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
      <div>TODO register player</div>
      <player-seeder />
      <div>TODO reset bracket</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import BracketForm from '@/components/BracketForm.vue';
import PlayerSeeder from '@/components/PlayerSeeder.vue'
import { onMounted } from 'vue'
import { ref } from 'vue'

// https://github.com/SortableJS/vue.draggable.next/blob/master/example/components/simple.vue
onMounted(async () => {

});

const list = ref([
  {id: 1, name: 'jean'},
  {id: 2, name: 'jean2'},
])

const dragging = ref(false)
const enabled = ref(true)
const submittedBracketName = ref('')

function registerBracket(n: string): void {
  submittedBracketName.value = n
}
</script>
