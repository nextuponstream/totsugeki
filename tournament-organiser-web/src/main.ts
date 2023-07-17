import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import SubmitBtnVue from './components/SubmitBtn.vue'
import en from './locales/en'
import fr from './locales/fr'
import { createI18n } from 'vue-i18n'

const app = createApp(App)

const i18n = createI18n({
    locale: "en",
    fallbackLocale: "en",
    messages: { en, fr },
});

app.use(i18n)

app.use(createPinia())
app.use(router)

app.component('SubmitBtn', SubmitBtnVue)

app.mount('#app')
