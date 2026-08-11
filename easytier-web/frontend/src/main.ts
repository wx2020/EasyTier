import { createApp } from 'vue'
import 'easytier-frontend-lib/style.css'
import './style.css'
import App from './App.vue'
import EasytierFrontendLib from 'easytier-frontend-lib'
import PrimeVue from 'primevue/config'
import Aura from '@primeuix/themes/aura';
import ConfirmationService from 'primevue/confirmationservice';
import { I18nUtils } from 'easytier-frontend-lib'

import { createRouter, createWebHashHistory } from 'vue-router'
import DialogService from 'primevue/dialogservice';
import ToastService from 'primevue/toastservice';
import ConfigGenerator from './components/ConfigGenerator.vue'

const routes = [
    {
        path: '/auth', children: [
            {
                name: 'login',
                path: '',
                component: () => import('./components/Login.vue'),
                alias: 'login',
                props: { isRegistering: false }
            },
            {
                name: 'register',
                path: 'register',
                component: () => import('./components/Login.vue'),
                props: { isRegistering: true }
            }
        ]
    },
    {
        path: '/h/:apiHost', component: () => import('./components/MainPage.vue'), children: [
            {
                path: '',
                alias: 'dashboard',
                name: 'dashboard',
                component: () => import('./components/Dashboard.vue'),
            },
            {
                path: 'deviceList',
                name: 'deviceList',
                component: () => import('./components/DeviceList.vue'),
                children: [
                    {
                        path: 'device/:deviceId/:instanceId?',
                        name: 'deviceManagement',
                        component: () => import('./components/DeviceManagement.vue'),
                    }
                ]
            },
        ]
    },
    {
        path: '/:pathMatch(.*)*', name: 'notFound', redirect: () => {
            let apiHost = localStorage.getItem('apiHost');
            if (apiHost) {
                return { name: 'dashboard', params: { apiHost: apiHost } }
            } else {
                return { name: 'login' }
            }
        }
    },
    {
        path: '/config_generator',
        component: ConfigGenerator,
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

const app = createApp(App)

// Use i18n
app.use(I18nUtils.i18n)

app.use(PrimeVue,
    {
        theme: {
            preset: Aura,
            options: {
                prefix: 'p',
                darkModeSelector: 'system',
                cssLayer: {
                    name: 'primevue',
                    order: 'tailwind-base, primevue, tailwind-utilities'
                }
            }
        }
    }
).use(ToastService as any).use(DialogService as any).use(router).use(ConfirmationService as any).use(EasytierFrontendLib).mount('#app')
