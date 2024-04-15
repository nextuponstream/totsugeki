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
    :sort-order="-1"
    sort-field="created_at"
    :rows-per-page-options="[10, 25, 50, 100]"
    table-style="min-width: 50rem"
    :current-page-report-template="`{first} ${$t(
      'paginator.pageReport'
    )} {last}`"
    @page="paginatorUpdate"
    @sort="sortEvent"
  >
    <Column field="name" :header="$t('generic.name')">
      <template #body="slotProps">
        <a
          :href="`/brackets/${slotProps.data.id}`"
          :data-test-id="slotProps.data.id"
          style="color: blue; text-decoration: underline"
          >{{ nameFallback(slotProps.data.name) }}</a
        >
      </template>
    </Column>
    <Column
      field="created_at"
      :header="$t('generic.created_at')"
      :sortable="true"
    ></Column>
  </DataTable>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useUserStore } from '@/stores/user'
import { useBracketStore } from '@/stores/bracket'
import { nameFallback } from '@/helpers'

const userStore = useUserStore()
const bracketStore = useBracketStore()

import DataTable, {
  type DataTablePageEvent,
  type DataTableSortEvent,
} from 'primevue/datatable'
import Column from 'primevue/column'

onMounted(async () => {
  await bracketStore.getBracketsFrom(userStore.id!)
})

function paginatorUpdate(_e: DataTablePageEvent) {
  // console.debug(JSON.stringify(_e))
  bracketStore.getBracketsFrom(userStore.id!)
}

function sortEvent(e: DataTableSortEvent) {
  // NOTE: page number is reset. It might be annoying?
  // console.debug(JSON.stringify(e))
  if (e.sortField === 'created_at') {
    if (e.sortOrder === 1) {
      bracketStore.pagination.sortOrder = 'ASC'
    } else if (e.sortOrder === -1) {
      bracketStore.pagination.sortOrder = 'DESC'
    }
  }
  bracketStore.getBracketsFrom(userStore.id!)
}
</script>
<style scoped></style>
