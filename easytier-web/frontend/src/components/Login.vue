<script setup lang="ts">
import { I18nUtils } from 'easytier-frontend-lib'
import { AutoComplete, Button, Card, InputText, Password } from 'primevue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useToast } from 'primevue/usetoast'

import { cleanAndLoadApiHosts, getInitialApiHost, saveApiHost } from '../modules/api-host'
import ApiClient, { type Credential, type OidcConfigResponse, type RegisterData } from '../modules/api'

const props = defineProps<{
  isRegistering: boolean
}>()

const { t } = useI18n()
const router = useRouter()
const toast = useToast()

const apiHost = ref(getInitialApiHost())
const apiHostSuggestions = ref<string[]>([])
const api = computed(() => new ApiClient(apiHost.value))

const username = ref('')
const password = ref('')
const registerUsername = ref('')
const registerPassword = ref('')
const captcha = ref('')
const captchaVersion = ref(Date.now())
const captchaSrc = computed(() => `${api.value.captcha_url()}?v=${captchaVersion.value}`)

const passwordSubmitting = ref(false)
const registrationSubmitting = ref(false)
const ssoRedirecting = ref(false)

type OidcState = 'idle' | 'loading' | 'enabled' | 'disabled' | 'error'
const oidcState = ref<OidcState>('idle')
const oidcConfig = ref<OidcConfigResponse | null>(null)
const oidcCheckTimer = ref<ReturnType<typeof setTimeout> | null>(null)
let oidcRequestVersion = 0

const providerName = computed(() => oidcConfig.value?.provider_name || t('web.login.sso_provider'))
const ssoButtonLabel = computed(() => t('web.login.continue_with', { provider: providerName.value }))

function persistHost() {
  saveApiHost(apiHost.value)
  localStorage.setItem('apiHost', btoa(apiHost.value))
}

async function onSubmit() {
  passwordSubmitting.value = true
  persistHost()
  try {
    const credential: Credential = { username: username.value, password: password.value }
    const result = await api.value.login(credential)
    if (!result.success) {
      toast.add({ severity: 'error', summary: t('web.login.login_failed'), detail: result.message, life: 3000 })
      return
    }
    await router.push({ name: 'dashboard', params: { apiHost: btoa(apiHost.value) } })
  }
  finally {
    passwordSubmitting.value = false
  }
}

async function onRegister() {
  registrationSubmitting.value = true
  persistHost()
  try {
    const credentials: Credential = {
      username: registerUsername.value,
      password: registerPassword.value,
    }
    const request: RegisterData = { credentials, captcha: captcha.value }
    const result = await api.value.register(request)
    if (!result.success) {
      captchaVersion.value = Date.now()
      toast.add({ severity: 'error', summary: t('web.login.register_failed'), detail: result.message, life: 3000 })
      return
    }
    toast.add({ severity: 'success', summary: t('web.login.register_success'), detail: result.message, life: 2500 })
    await router.push({ name: 'login' })
  }
  finally {
    registrationSubmitting.value = false
  }
}

function searchApiHosts(event: { query: string }) {
  const candidates = cleanAndLoadApiHosts().map(host => host.value)
  apiHostSuggestions.value = [...new Set([event.query, ...candidates].filter(Boolean))]
}

function scheduleOidcCheck() {
  if (oidcCheckTimer.value)
    clearTimeout(oidcCheckTimer.value)

  const requestVersion = ++oidcRequestVersion
  oidcState.value = 'loading'
  oidcConfig.value = null
  oidcCheckTimer.value = setTimeout(async () => {
    try {
      const config = await new ApiClient(apiHost.value).getOidcConfig()
      if (requestVersion !== oidcRequestVersion)
        return
      oidcConfig.value = config
      oidcState.value = config.enabled ? 'enabled' : 'disabled'
    }
    catch {
      if (requestVersion === oidcRequestVersion)
        oidcState.value = 'error'
    }
  }, 300)
}

function onSsoLogin() {
  if (oidcState.value !== 'enabled')
    return
  persistHost()
  ssoRedirecting.value = true
  window.location.assign(api.value.oidcLoginUrl())
}

function goTo(routeName: 'login' | 'register') {
  saveApiHost(apiHost.value)
  void router.replace({ name: routeName })
}

watch(apiHost, scheduleOidcCheck)
onMounted(scheduleOidcCheck)
onBeforeUnmount(() => {
  oidcRequestVersion++
  if (oidcCheckTimer.value)
    clearTimeout(oidcCheckTimer.value)
})
</script>

<template>
  <main class="auth-shell">
    <Button
      icon="pi pi-language"
      type="button"
      class="language-button"
      severity="secondary"
      rounded
      :aria-label="t('web.main.language')"
      @click="I18nUtils.toggleLanguage"
    />

    <section class="auth-layout" aria-labelledby="auth-title">
      <aside class="brand-panel">
        <span class="brand-mark">ET</span>
        <div>
          <p class="eyebrow">EasyTier Web</p>
          <h1>{{ t('web.login.welcome') }}</h1>
          <p>{{ t('web.login.subtitle') }}</p>
        </div>
        <div class="security-note">
          <i class="pi pi-shield" aria-hidden="true" />
          <span>{{ t('web.login.secure_auth') }}</span>
        </div>
      </aside>

      <Card class="auth-card">
        <template #content>
          <header class="form-header">
            <p class="eyebrow">{{ props.isRegistering ? t('web.login.create_account') : t('web.login.account_access') }}</p>
            <h2 id="auth-title">{{ props.isRegistering ? t('web.login.register') : t('web.login.login') }}</h2>
          </header>

          <div class="field-group">
            <label for="api-host">{{ t('web.login.api_host') }}</label>
            <AutoComplete
              id="api-host"
              v-model="apiHost"
              dropdown
              required
              :suggestions="apiHostSuggestions"
              class="w-full"
              :placeholder="t('web.login.api_host_placeholder')"
              @complete="searchApiHosts"
            />
            <small>{{ t('web.login.api_host_hint') }}</small>
          </div>

          <template v-if="!props.isRegistering">
            <div v-if="oidcState === 'enabled' || oidcState === 'loading'" class="sso-section" aria-live="polite">
              <Button
                icon="pi pi-shield"
                type="button"
                class="w-full justify-center"
                severity="contrast"
                :label="oidcState === 'loading' ? t('web.login.checking_sso') : ssoButtonLabel"
                :loading="oidcState === 'loading' || ssoRedirecting"
                :disabled="oidcState !== 'enabled' || ssoRedirecting"
                @click="onSsoLogin"
              />
              <div class="auth-divider"><span>{{ t('web.login.or_password') }}</span></div>
            </div>
            <p v-else-if="oidcState === 'error'" class="status-message" role="status">
              {{ t('web.login.sso_unavailable') }}
            </p>

            <form class="form-stack" @submit.prevent="onSubmit">
              <div class="field-group">
                <label for="username">{{ t('web.login.username') }}</label>
                <InputText id="username" v-model="username" required autocomplete="username" class="w-full" />
              </div>
              <div class="field-group">
                <label for="password">{{ t('web.login.password') }}</label>
                <Password id="password" v-model="password" required autocomplete="current-password" toggle-mask :feedback="false" />
              </div>
              <Button type="submit" class="w-full justify-center" :label="t('web.login.login')" :loading="passwordSubmitting" />
              <Button type="button" class="w-full justify-center" severity="secondary" variant="text" :label="t('web.login.register')" @click="goTo('register')" />
            </form>
          </template>

          <form v-else class="form-stack" @submit.prevent="onRegister">
            <div class="field-group">
              <label for="register-username">{{ t('web.login.username') }}</label>
              <InputText id="register-username" v-model="registerUsername" required autocomplete="username" class="w-full" />
            </div>
            <div class="field-group">
              <label for="register-password">{{ t('web.login.password') }}</label>
              <Password id="register-password" v-model="registerPassword" required autocomplete="new-password" toggle-mask :feedback="false" />
            </div>
            <div class="field-group">
              <label for="captcha">{{ t('web.login.captcha') }}</label>
              <div class="captcha-row">
                <InputText id="captcha" v-model="captcha" required autocomplete="off" class="w-full" />
                <button type="button" class="captcha-button" :aria-label="t('web.login.refresh_captcha')" @click="captchaVersion = Date.now()">
                  <img :src="captchaSrc" :alt="t('web.login.captcha_alt')">
                </button>
              </div>
            </div>
            <Button type="submit" class="w-full justify-center" :label="t('web.login.register')" :loading="registrationSubmitting" />
            <Button type="button" class="w-full justify-center" severity="secondary" variant="text" :label="t('web.login.back_to_login')" @click="goTo('login')" />
          </form>
        </template>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.auth-shell {
  min-height: 100dvh;
  display: grid;
  place-items: center;
  padding: clamp(1rem, 4vw, 3rem);
  color: var(--p-text-color);
  background:
    radial-gradient(circle at 12% 18%, color-mix(in srgb, var(--p-primary-400) 24%, transparent), transparent 38%),
    radial-gradient(circle at 88% 82%, color-mix(in srgb, var(--p-primary-700) 20%, transparent), transparent 36%),
    var(--p-surface-ground);
}

.language-button {
  position: fixed;
  z-index: 10;
  top: max(1rem, env(safe-area-inset-top));
  right: max(1rem, env(safe-area-inset-right));
}

.auth-layout {
  width: min(100%, 960px);
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(360px, 440px);
  overflow: hidden;
  border: 1px solid var(--p-surface-border);
  border-radius: 1.5rem;
  background: color-mix(in srgb, var(--p-surface-card) 92%, transparent);
  box-shadow: 0 28px 80px rgb(15 23 42 / 18%);
  backdrop-filter: blur(18px);
}

.brand-panel {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 580px;
  padding: clamp(2rem, 5vw, 4rem);
  color: white;
  background: linear-gradient(145deg, var(--p-primary-700), var(--p-primary-500));
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 3rem;
  height: 3rem;
  border: 1px solid rgb(255 255 255 / 48%);
  border-radius: 1rem;
  font-weight: 800;
  letter-spacing: .08em;
  background: rgb(255 255 255 / 14%);
}

.brand-panel h1 {
  max-width: 12ch;
  margin: .5rem 0 1rem;
  font-size: clamp(2rem, 4vw, 3.5rem);
  line-height: 1.05;
}

.brand-panel p {
  max-width: 42ch;
  color: rgb(255 255 255 / 80%);
}

.eyebrow {
  margin: 0;
  font-size: .75rem;
  font-weight: 700;
  letter-spacing: .14em;
  text-transform: uppercase;
}

.security-note {
  display: flex;
  gap: .75rem;
  align-items: center;
  font-size: .875rem;
  color: rgb(255 255 255 / 82%);
}

.auth-card {
  border: 0;
  border-radius: 0;
  box-shadow: none;
}

.form-header {
  margin-bottom: 1.75rem;
}

.form-header .eyebrow {
  color: var(--p-primary-color);
}

.form-header h2 {
  margin: .35rem 0 0;
  font-size: 1.75rem;
}

.form-stack,
.field-group {
  display: grid;
  gap: .5rem;
}

.form-stack {
  gap: 1rem;
}

.field-group {
  margin-bottom: 1rem;
}

.field-group label {
  font-size: .875rem;
  font-weight: 650;
}

.field-group small,
.status-message {
  color: var(--p-text-muted-color);
}

.sso-section {
  margin: 1.25rem 0;
}

.auth-divider {
  display: flex;
  align-items: center;
  gap: .75rem;
  margin: 1.25rem 0;
  color: var(--p-text-muted-color);
  font-size: .75rem;
}

.auth-divider::before,
.auth-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--p-surface-border);
}

.status-message {
  margin: 0 0 1rem;
  font-size: .8rem;
}

.captcha-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: .75rem;
  align-items: center;
}

.captcha-button {
  overflow: hidden;
  padding: 0;
  border: 1px solid var(--p-surface-border);
  border-radius: .5rem;
  cursor: pointer;
  background: var(--p-surface-50);
}

.captcha-button img {
  display: block;
  width: 127px;
  height: 48px;
}

@media (max-width: 760px) {
  .auth-shell {
    padding: 0;
    place-items: stretch;
  }

  .auth-layout {
    min-height: 100dvh;
    grid-template-columns: 1fr;
    border: 0;
    border-radius: 0;
  }

  .brand-panel {
    min-height: auto;
    padding: 2rem 1.5rem;
  }

  .brand-panel > div,
  .security-note {
    display: none;
  }

  .auth-card {
    align-self: stretch;
  }
}
</style>
