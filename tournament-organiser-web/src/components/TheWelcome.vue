<script lang="ts">
import WelcomeItem from './WelcomeItem.vue'
import DocumentationIcon from './icons/IconDocumentation.vue'
import ToolingIcon from './icons/IconTooling.vue'
import EcosystemIcon from './icons/IconEcosystem.vue'
import CommunityIcon from './icons/IconCommunity.vue'
import SupportIcon from './icons/IconSupport.vue'
import draggable from 'vuedraggable'

// https://github.com/SortableJS/vue.draggable.next/blob/master/example/components/simple.vue
export default {
  components: {
    draggable,
  },
  data() {
    return {
      list: [
        {id:1, name: "jean"},
        {id:2, name: "jean2"},
      ],
      dragging: false,
      enabled: true,
    }
  },
}
</script>

<template>
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
          <div class="list-group-item" :class="{ 'not-draggable': !enabled }">
            {{ element.name }}
          </div>
        </template>
      </draggable>
  </p>
</template>
