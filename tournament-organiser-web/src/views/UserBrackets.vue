<template>
  <DataTable
    v-model:first="bracketStore.pagination.offset"
    v-model:rows="bracketStore.pagination.limit"
    v-model:total-records="bracketStore.pagination.total"
    style="font-size: 11px"
    :value="bracketStore.bracketList"
    paginator
    lazy
    paginator-template="RowsPerPageDropdown FirstPageLink PrevPageLink CurrentPageReport NextPageLink LastPageLink"
    :striped-rows="true"
    :rows-per-page-options="[10, 25, 50, 100]"
    table-style="min-width: 50rem"
    @page="paginatorUpdate"
  >
    <Column field="name" header="Name">
      <template #body="slotProps">
        <a
          :href="`/brackets/${slotProps.data.id}`"
          :data-test-id="slotProps.data.id"
          style="color: blue; text-decoration: underline"
          >{{ nameFallback(slotProps.data.name) }}</a
        >
      </template>
    </Column>
    <Column field="created_at" header="Created at"></Column>
  </DataTable>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useUserStore } from '@/stores/user'
import { useBracketStore } from '@/stores/bracket'
import { nameFallback } from '@/helpers'

const userStore = useUserStore()
const bracketStore = useBracketStore()

import DataTable, { type DataTablePageEvent } from 'primevue/datatable'
import Column from 'primevue/column'

onMounted(async () => {
  await bracketStore.getBracketsFrom(userStore.id!)
})

function paginatorUpdate(_e: DataTablePageEvent) {
  bracketStore.getBracketsFrom(userStore.id!)
}
</script>
<style scoped></style>
