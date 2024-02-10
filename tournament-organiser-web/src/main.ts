import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import en from './locales/en'
import fr from './locales/fr'
import { createI18n } from 'vue-i18n'

import FormInputVue from './components/ui/FormInput.vue'
import SubmitBtnVue from './components/ui/SubmitBtn.vue'
import BaseTooltip from './components/ui/BaseTooltip.vue'
import CancelBtnVue from './components/ui/CancelBtn.vue'
import OtherBtnVue from './components/ui/OtherBtn.vue'
import BaseLink from './components/ui/BaseLink.vue'
import DangerBtnVue from './components/ui/DangerBtn.vue'
import BaseModalVue from './components/ui/BaseModal.vue'

const app = createApp(App)

const i18n = createI18n({
  locale: 'en',
  fallbackLocale: 'en',
  messages: { en, fr },
})

app.use(i18n)

app.use(createPinia())
app.use(router)

app.component('SubmitBtn', SubmitBtnVue)
app.component('CancelBtn', CancelBtnVue)
app.component('OtherBtn', OtherBtnVue)
app.component('DangerBtn', DangerBtnVue)
app.component('FormInput', FormInputVue)
app.component('BaseTooltip', BaseTooltip)
app.component('BaseLink', BaseLink)
app.component('BaseModal', BaseModalVue)

app.mount('#app')
