import type { InjectionKey, Ref } from 'vue'

export const BASE_URL = import.meta.env.VITE_BASE_URL ?? 'http://localhost:5173'
export const showMenuKey = Symbol() as InjectionKey<Ref<Boolean | null>>
