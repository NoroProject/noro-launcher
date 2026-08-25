<script setup lang="ts">
const route = useRoute()
const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const busy = ref(false)
const message = ref<string | null>(null)
const showRecovery = ref(false)
const recoveryUser = ref('')
const recoveryCode = ref('')

const next = computed(() => String(route.query.next || '/cabinet'))

// Кнопки строятся по тому, что инстанс реально умеет: показывать вход через
// платформу, которую оператор не настроил, значит вести игрока в отказ.
const methods = ref<AuthMethods>({ providers: [], passkey: false })
const loginHref = (provider: string) =>
  auth.providerLoginUrl(provider, `/login?next=${encodeURIComponent(next.value)}`)

// Картинка страницы входа задаётся в админке — своя, не та же, что на главной.
// Запрос идёт по адресу мастера, а не относительным путём: сайт и API живут на
// одном домене только за общим прокси, а в разработке это разные порты.
const loginImage = ref('')
const imageFailed = ref(false)


async function loginWithPasskey() {
  busy.value = true
  message.value = null
  try {
    const opts = await auth.request<ChallengeRes>('/api/auth/passkeys/login/options', { method: 'POST' })
    const credential = await getCredential(opts)

    const res = await auth.request<any>('/api/auth/passkeys/login/verify', {
      method: 'POST',
      body: { state_id: opts.state_id, credential }
    })
    auth.token.value = res.access_token
    auth.user.value = res.user
    await navigateTo(next.value)
  } catch (e) {
    message.value = humanError(e)
  } finally {
    busy.value = false
  }
}

async function loginWithRecovery() {
  busy.value = true
  message.value = null
  try {
    const res = await auth.request<any>('/api/auth/recovery/login', {
      method: 'POST',
      body: { username: recoveryUser.value.trim(), code: recoveryCode.value.trim() },
    })
    auth.token.value = res.access_token
    auth.user.value = res.user
    // Код сгорел — следующий вход должен опираться на что-то долговечнее.
    await navigateTo(res.bind_passkey ? '/cabinet?bind_passkey=1' : next.value)
  } catch (e) {
    message.value = humanError(e)
  } finally {
    busy.value = false
  }
}

onMounted(async () => {
  methods.value = await loadAuthMethods()
  loginImage.value = await loadLoginImage()

  const access = route.query.access_token
  const refresh = route.query.refresh_token
  if (typeof access !== 'string') {
    if (auth.token.value) await auth.loadMe()
    return
  }

  busy.value = true
  auth.token.value = access
  auth.refreshToken.value = typeof refresh === 'string' ? refresh : null
  const user = await auth.loadMe()
  busy.value = false
  if (user) {
    await navigateTo(next.value)
  } else {
    message.value = auth.error.value || 'Token received, but profile failed to load'
  }
})
</script>

<template>
  <div class="grid min-h-screen place-items-center px-4">
    <!-- Высота панели задана и не зависит от того, раскрыта ли форма кода:
         иначе карточка подпрыгивала на полтораста пикселей, а вместе с ней
         уезжала и картинка справа. -->
    <div class="noro-panel grid w-full max-w-5xl overflow-hidden p-0 md:h-[600px] md:grid-cols-[420px_1fr]">
      <section class="noro-scroll flex flex-col gap-8 overflow-y-auto p-8 md:p-10">
        <header class="flex items-center justify-between gap-3">
          <div class="flex min-w-0 items-center gap-2.5">
            <img src="/icon.png" alt="" class="size-8 shrink-0 rounded">
            <span class="noro-pixel truncate text-base uppercase text-[var(--noro-cream)]">NORO</span>
          </div>
          <LocaleSwitch />
        </header>

        <div class="flex flex-1 flex-col justify-center gap-5">
          <div>
            <h1 class="noro-pixel text-2xl leading-tight text-[var(--noro-cream)]">{{ t('login-heading') }}</h1>
            <p class="mt-2 text-xs leading-5 text-[var(--noro-muted)]">
              {{ showRecovery ? t('login-recovery-lead') : t('login-lead') }}
            </p>
          </div>

          <UAlert
            v-if="message || auth.error.value"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="message || auth.error.value || undefined"
          />

          <!-- Либо кнопки платформ, либо форма кода — не то и другое разом:
               вместе это читалось как две несвязанные формы на одном экране. -->
          <div v-if="!showRecovery" class="space-y-2">
            <!-- Логотип платформы на кнопке обязателен по её бренд-гайдлайну,
                 и одинаковый вид у всех кнопок — тоже требование: Google не
                 разрешает делать свой вход менее заметным, чем чужой. -->
            <NuxtLink
              v-for="p in methods.providers"
              :key="p.provider"
              :to="loginHref(p.provider)"
              class="flex h-12 w-full items-center gap-3 rounded-[var(--noro-r-md)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 text-xs font-bold text-[var(--noro-text)] transition hover:border-[var(--noro-cream)] hover:bg-[var(--noro-panel-2)]"
            >
              <UIcon
                :name="providerMeta(p.provider).icon"
                class="size-5 shrink-0"
                :style="providerIconStyle(p.provider)"
              />
              <span class="truncate">
                {{ busy || auth.loading.value ? t('login-waiting') : t('login-with', { provider: p.name }) }}
              </span>
              <UIcon name="i-lucide-arrow-right" class="ml-auto size-4 shrink-0 opacity-30" />
            </NuxtLink>

            <UAlert
              v-if="!methods.providers.length && !methods.passkey"
              color="warning"
              variant="subtle"
              icon="i-lucide-triangle-alert"
              :description="t('login-no-methods')"
            />

            <button
              v-if="methods.passkey"
              type="button"
              class="flex h-12 w-full items-center gap-3 rounded-[var(--noro-r-md)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 text-xs font-bold text-[var(--noro-text)] transition hover:border-[var(--noro-cream)] hover:bg-[var(--noro-panel-2)]"
              :disabled="busy"
              @click="loginWithPasskey"
            >
              <UIcon name="i-lucide-key-round" class="size-5 shrink-0 text-[var(--noro-blue)]" />
              <span class="truncate">{{ t('login-passkey') }}</span>
              <UIcon name="i-lucide-arrow-right" class="ml-auto size-4 shrink-0 opacity-30" />
            </button>
          </div>

          <!-- Не «аварийный вариант»: passkey нельзя привязать по http:// без
               домена, и до настройки TLS код — единственный путь внутрь. -->
          <form v-if="showRecovery" class="grid gap-2" @submit.prevent="loginWithRecovery">
            <input v-model="recoveryUser" class="noro-input w-full" :placeholder="t('login-username-placeholder')" autocomplete="username">
            <input v-model="recoveryCode" class="noro-input w-full font-mono" placeholder="XXXX-XXXX-XXXX">
            <button
              type="submit"
              class="noro-cta w-full px-6 py-3 text-center text-xs font-bold"
              :disabled="busy || !recoveryUser.trim() || !recoveryCode.trim()"
            >
              {{ t('login-submit') }}
            </button>
            <p class="text-[10px] leading-4 text-[var(--noro-muted)]">
              {{ t('login-recovery-hint') }}
            </p>
          </form>

          <!-- Дорога назад обязательна: без неё выйти из формы кода можно
               только перезагрузкой страницы. -->
          <button
            type="button"
            class="flex items-center gap-1.5 self-start text-[11px] font-bold uppercase tracking-wider text-[var(--noro-muted)] transition hover:text-[var(--noro-cream)]"
            @click="showRecovery = !showRecovery"
          >
            <UIcon v-if="showRecovery" name="i-lucide-arrow-left" class="size-3.5" />
            {{ showRecovery ? t('login-recovery-back') : t('login-recovery-toggle') }}
          </button>
        </div>
      </section>

      <!-- Иллюстрация инстанса вместо набора фигур. Пусто — спокойная
           заглушка: страница не должна выглядеть сломанной у того, кто ещё
           ничего не загрузил. -->
      <section class="relative hidden min-h-[520px] overflow-hidden bg-[var(--noro-bg-deep)] md:block">
        <!-- Картинка не открылась (сменили CDN, файл удалён, домен закрыт) —
             показываем ту же заглушку, а не иконку битого изображения. -->
        <img
          v-if="loginImage && !imageFailed"
          :src="loginImage"
          alt=""
          class="size-full object-cover"
          @error="imageFailed = true"
        >
        <div v-else class="grid size-full place-items-center">
          <div class="flex flex-col items-center gap-3 text-[var(--noro-muted)]">
            <UIcon name="i-lucide-image" class="size-10 opacity-40" />
            <span class="text-[11px] uppercase tracking-wider opacity-60">{{ t('login-image-empty') }}</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
