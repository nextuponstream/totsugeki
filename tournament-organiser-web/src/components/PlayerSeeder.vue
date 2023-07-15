<template>
  <div class="text-3xl">
    Seeder
  </div>
  <p>
    <draggable 
      :list="list" 
      :disabled="!enabled"
      item-key="name"
      class="list-group"
      ghost-class="ghost"
      @start="dragging = true"
      @end="dragging = false;console.log(JSON.stringify(list))"
    >
      <template #item="{ element }">
        <div
          class="list-group-item"
          :class="{ 'not-draggable': !enabled }"
        >
          {{ element.name }}
        </div>
      </template>
    </draggable>
  </p>
</template>
  
  <script setup lang="ts">
  import draggable from 'vuedraggable'
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
  