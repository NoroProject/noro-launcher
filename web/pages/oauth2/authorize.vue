<script setup lang="ts">
/**
 * Экран согласия: единственное место, где игрок решает, что отдать приложению.
 *
 * Показываем то, что решает: чьё приложение, что именно оно получит и куда
 * вернётся. Раньше здесь была заглушка — любой `client_id` рисовался как
 * «официальное приложение» с одной строкой про профиль, потому что данные брались
 * из ручки, которая на самом деле отдаёт редирект и всегда падала в catch.
 */

const route = useRoute()
const auth = useAuth()
const { t } = useT()

definePageMeta({ layout: false })

const clientId = computed(() => String(route.query.client_id || ''))
const redirectUri = computed(() => String(route.query.redirect_uri || ''))
const scope = computed(() => String(route.query.scope || ''))
const state = computed(() => String(route.query.state || ''))
const challenge = computed(() => String(route.query.code_challenge || ''))

interface ConsentScope { name: string; title: string; tier: 'basic' | 'privileged' }
interface Consent {
  name: string
  icon_url: string | null
  description: string | null
  official: boolean
  owner: string | null
  status: string
  usable: boolean
  scopes: ConsentScope[]
}

const consent = ref<Consent | null>(null)
const busy = ref(true)
const errorMsg = ref<string | null>(null)

/** Куда вернётся игрок. Домен, а не весь адрес: он и решает, кому доверять. */
const returnHost = computed(() => {
  try {
    return new URL(redirectUri.value).host
  } catch {
    return redirectUri.value
  }
})

/** Значок доступа. Привилегированные заметно строже — их выдаёт оператор. */
function scopeIcon(s: ConsentScope) {
  const icons: Record<string, string> = {
    identity: 'i-lucide-user',
    profile: 'i-lucide-id-card',
    skins: 'i-lucide-shirt',
    capes: 'i-lucide-flag',
    punishments: 'i-lucide-gavel',
    servers: 'i-lucide-server',
    'skins:write': 'i-lucide-pencil',
    identities: 'i-lucide-link',
    journal: 'i-lucide-scroll-text',
    launcher: 'i-lucide-shield-alert',
  }
  return icons[s.name] || 'i-lucide-check'
}

/** Перевод доступа, если он есть; иначе формулировка мастера. */
const scopeText = (s: ConsentScope) => {
  const key = `oauth-scope-${s.name.replace(':', '-')}`
  const translated = t(key)
  return translated === key ? s.title : translated
}

onMounted(async () => {
  if (!clientId.value || !redirectUri.value) {
    busy.value = false
    errorMsg.value = t('oauth-missing-params')
    return
  }
  if (!auth.token.value) {
    await navigateTo(link.login(route.fullPath))
    return
  }
  if (!auth.user.value) await auth.loadMe()

  try {
    consent.value = await auth.request<Consent>('/api/oauth2/consent', {
      query: {
        client_id: clientId.value,
        redirect_uri: redirectUri.value,
        scope: scope.value,
      },
    })
  } catch (e) {
    errorMsg.value = apiErrorMessage(e) || String(e)
  } finally {
    busy.value = false
  }
})

async function allow() {
  busy.value = true
  errorMsg.value = null
  try {
    const res = await auth.request<{ redirect: string }>('/api/oauth2/authorize/accept', {
      method: 'POST',
      body: {
        client_id: clientId.value,
        redirect_uri: redirectUri.value,
        scopes: scope.value,
        state: state.value,
        code_challenge: challenge.value || null,
      },
    })
    window.location.href = res.redirect
  } catch (e) {
    errorMsg.value = apiErrorMessage(e) || String(e)
    busy.value = false
  }
}

/** Отказ возвращает приложение ни с чем — так требует OAuth 2.0. */
function deny() {
  const sep = redirectUri.value.includes('?') ? '&' : '?'
  window.location.href = `${redirectUri.value}${sep}error=access_denied&state=${encodeURIComponent(state.value)}`
}
</script>

<template>
  <div class="grid min-h-screen place-items-center bg-[var(--noro-bg-deep)] px-4 py-10">
    <div class="w-full max-w-md">
      <div class="noro-panel p-8">
        <div v-if="busy && !consent" class="flex flex-col items-center gap-4 py-12">
          <UIcon name="i-lucide-loader-2" class="size-7 animate-spin text-[var(--noro-blue)]" />
          <span class="text-xs text-[var(--noro-muted)]">{{ t('oauth-loading-app') }}</span>
        </div>

        <template v-else-if="consent">
          <!-- Кто и кому: приложение слева, игрок справа. -->
          <div class="flex items-center justify-center gap-4">
            <div class="grid size-16 place-items-center overflow-hidden rounded-2xl border border-[var(--noro-border)] bg-[var(--noro-panel-2)]">
              <img v-if="consent.icon_url" :src="consent.icon_url" alt="" class="size-full object-cover">
              <UIcon v-else name="i-lucide-app-window" class="size-7 text-[var(--noro-muted)]" />
            </div>
            <UIcon name="i-lucide-arrow-right" class="size-4 text-[var(--noro-muted)]" />
            <!-- Аватар платформы, которой игрок вошёл. Раньше здесь стоял
                 `/api/avatar/{ник}` — такой ручки нет ни у сайта, ни у мастера,
                 и картинка всегда молча падала в запасной значок. -->
            <div class="grid size-16 place-items-center overflow-hidden rounded-2xl border border-[var(--noro-border)] bg-[var(--noro-panel-2)]">
              <img
                v-if="identityAvatar(auth.user.value)"
                :src="identityAvatar(auth.user.value)!"
                alt=""
                class="size-full object-cover"
              >
              <span v-else class="text-xl font-black text-[var(--noro-cream)]">
                {{ (auth.user.value?.username || '?').slice(0, 1).toUpperCase() }}
              </span>
            </div>
          </div>

          <h1 class="mt-6 text-center text-xl font-black text-[var(--noro-cream)]">
            {{ consent.name }}
          </h1>
          <p class="mt-1 text-center text-sm text-[var(--noro-muted)]">
            {{ t('oauth-wants-access', { user: auth.user.value?.username || '' }) }}
          </p>

          <div class="mt-3 flex flex-wrap items-center justify-center gap-2">
            <span
              v-if="consent.official"
              class="inline-flex items-center gap-1 rounded-full bg-[color-mix(in_srgb,var(--noro-blue)_18%,transparent)] px-2.5 py-1 text-[10px] font-black uppercase tracking-wider text-[var(--noro-blue)]"
            >
              <UIcon name="i-lucide-badge-check" class="size-3" />
              {{ t('oauth-official-app') }}
            </span>
            <span v-else-if="consent.owner" class="text-xs text-[var(--noro-muted)]">
              {{ t('oauth-by-developer', { owner: consent.owner }) }}
            </span>
          </div>

          <!-- Приложение ещё не проверено: автор так может отладить интеграцию,
               но игрок должен понимать, во что входит. -->
          <UAlert
            v-if="consent.status === 'pending'"
            class="mt-5"
            color="warning"
            variant="subtle"
            icon="i-lucide-clock"
            :description="t('oauth-app-pending')"
          />

          <div class="mt-6">
            <div class="noro-label mb-2">{{ t('oauth-requested-permissions') }}</div>
            <ul class="grid gap-2">
              <li
                v-for="s in consent.scopes"
                :key="s.name"
                class="flex items-start gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-input)] px-4 py-3"
              >
                <UIcon
                  :name="scopeIcon(s)"
                  class="mt-0.5 size-4 shrink-0"
                  :class="s.tier === 'privileged' ? 'text-[var(--noro-magenta)]' : 'text-[var(--noro-blue)]'"
                />
                <span class="text-xs leading-relaxed text-[var(--noro-text)]">{{ scopeText(s) }}</span>
              </li>
            </ul>
          </div>

          <p class="mt-4 text-center text-[11px] text-[var(--noro-muted)]">
            {{ t('oauth-will-return-to', { host: returnHost }) }}
          </p>

          <UAlert
            v-if="errorMsg"
            class="mt-5"
            color="error"
            variant="subtle"
            icon="i-lucide-triangle-alert"
            :description="errorMsg"
          />

          <div class="mt-6 grid grid-cols-2 gap-3">
            <AtomButton variant="secondary" :disabled="busy" @click="deny">
              {{ t('oauth-deny') }}
            </AtomButton>
            <AtomButton icon="i-lucide-check" :loading="busy" :disabled="!consent.usable" @click="allow">
              {{ t('oauth-allow') }}
            </AtomButton>
          </div>

          <p class="mt-4 text-center text-[11px] text-[var(--noro-muted)]">
            {{ t('oauth-revoke-hint') }}
          </p>
        </template>

        <!-- Приложение не найдено, адрес возврата чужой, доступ закрыт: всё это
             видно здесь, а не после молчаливого редиректа неизвестно куда. -->
        <template v-else>
          <div class="flex flex-col items-center gap-4 py-8 text-center">
            <UIcon name="i-lucide-shield-x" class="size-10 text-[var(--noro-danger)]" />
            <h1 class="text-lg font-black text-[var(--noro-cream)]">{{ t('oauth-cannot-continue') }}</h1>
            <p class="text-xs text-[var(--noro-muted)]">{{ errorMsg }}</p>
            <AtomButton variant="secondary" :to="link.home()">{{ t('oauth-back-home') }}</AtomButton>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
