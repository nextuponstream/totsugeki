<template>
  <bracket-form />
  <player-seeder />
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
