import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useLanguageStore = defineStore(
  'language',
  () => {
    const language = ref('en')
    const supportedLanguages = ref(['en', 'fr'])
    return {
      supportedLanguages,
      language,
    }
  },
  {}
)
