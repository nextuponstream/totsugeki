<template>
  <div
    class="relative group hover:text-gray-400 shrink-0"
    @click="copyUrlToClipboard"
  >
    {{ linkName }}
    <i class="pi pi-external-link" />
    <BaseTooltip>{{ t('externalLink.hover') }}</BaseTooltip>
  </div>
</template>

<script setup lang="ts">
import { useToastStore } from '@/stores/toast'
import { useI18n } from 'vue-i18n'
import BaseTooltip from '@/components/ui/BaseTooltip.vue'

const toastStore = useToastStore()
const { t } = useI18n({})

const props = defineProps({
  linkName: {
    type: String,
    default: () => '?',
  },
  link: {
    type: String,
    default: () => window.location.href, // default: current url
  },
})

async function copyUrlToClipboard() {
  await navigator.clipboard.writeText(props.link)
  toastStore.success(t('bracketView.copiedLinkToBracket'))
}
</script>
<style scoped></style>
