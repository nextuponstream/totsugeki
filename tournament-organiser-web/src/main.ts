import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import SubmitBtnVue from './components/SubmitBtn.vue'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.component('SubmitBtn', SubmitBtnVue)

app.mount('#app')
