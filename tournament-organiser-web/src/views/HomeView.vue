<template>
  <div class="px-2 pt-2">
    <div class="text-xl">
      Create a new bracket
    </div>
    <bracket-form />
    <player-seeder />
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

</script>
