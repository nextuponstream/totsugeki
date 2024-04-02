import type { InjectionKey, Ref } from 'vue'

export const showMenuKey = Symbol() as InjectionKey<Ref<Boolean | null>>
export const prefixKey = Symbol() as InjectionKey<string>
