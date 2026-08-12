<script setup lang="ts">
const route = useRoute()
const auth = useAuth()

definePageMeta({
  layout: false
})

const clientId = computed(() => String(route.query.client_id || ''))
const redirectUri = computed(() => String(route.query.redirect_uri || ''))
const scope = computed(() => String(route.query.scope || 'profile'))
const state = computed(() => String(route.query.state || ''))
const responseType = computed(() => String(route.query.response_type || 'code'))

const busy = ref(false)
const appInfo = ref<{
  id: string
  client_id: string
  name: string
  description: string | null
  icon_url: string | null
  is_official: boolean
} | null>(null)
const errorMsg = ref<string | null>(null)

onMounted(async () => {
  if (!clientId.value || !redirectUri.value) {
    errorMsg.value = 'Отсутствуют обязательные параметры: client_id и redirect_uri'
    return
  }

  // Check login status
  if (!auth.token.value) {
    const returnUrl = route.fullPath
    await navigateTo(`/login?next=${encodeURIComponent(returnUrl)}`)
    return
  }

  if (!auth.user.value) {
    await auth.loadMe()
  }

  busy.value = true
  try {
    const res = await auth.request<any>(`/oauth2/authorize?client_id=${encodeURIComponent(clientId.value)}&redirect_uri=${encodeURIComponent(redirectUri.value)}&scope=${encodeURIComponent(scope.value)}&state=${encodeURIComponent(state.value)}&response_type=${encodeURIComponent(responseType.value)}`)
    if (res && res.app) {
      appInfo.value = res.app
    } else {
      appInfo.value = {
        id: 'noro_launcher',
        client_id: clientId.value,
        name: clientId.value === 'noro_launcher' ? 'Noro Launcher' : clientId.value,
        description: 'Доступ к вашему игровому профилю и скинам Noro Network',
        icon_url: null,
        is_official: true
      }
    }
  } catch (e) {
    appInfo.value = {
      id: clientId.value,
      client_id: clientId.value,
      name: clientId.value === 'noro_launcher' ? 'Noro Launcher' : clientId.value,
      description: 'Доступ к вашему игровому профилю и скинам Noro Network',
      icon_url: null,
      is_official: true
    }
  } finally {
    busy.value = false
  }
})

async function onAuthorize() {
  busy.value = true
  errorMsg.value = null
  try {
    const res = await auth.request<any>('/oauth2/authorize/accept', {
      method: 'POST',
      body: {
        client_id: clientId.value,
        redirect_uri: redirectUri.value,
        scopes: scope.value,
        state: state.value
      }
    })

    if (res && res.redirect) {
      window.location.href = res.redirect
    } else if (res && res.code) {
      const sep = redirectUri.value.includes('?') ? '&' : '?'
      window.location.href = `${redirectUri.value}${sep}code=${res.code}&state=${encodeURIComponent(state.value)}`
    } else {
      const sep = redirectUri.value.includes('?') ? '&' : '?'
      window.location.href = `${redirectUri.value}${sep}code=authorized&state=${encodeURIComponent(state.value)}`
    }
  } catch (e: any) {
    errorMsg.value = e?.message || e?.data?.error || String(e)
    busy.value = false
  }
}

function onDeny() {
  const sep = redirectUri.value.includes('?') ? '&' : '?'
  window.location.href = `${redirectUri.value}${sep}error=access_denied&state=${encodeURIComponent(state.value)}`
}
</script>

<template>
  <div class="relative grid min-h-screen place-items-center overflow-hidden bg-[var(--noro-bg-deep)] px-4 py-8">
    <div class="pointer-events-none absolute left-1/2 top-1/4 size-[500px] -translate-x-1/2 rounded-full bg-[var(--noro-magenta)]/15 blur-[120px]" />
    <div class="pointer-events-none absolute bottom-1/4 right-1/4 size-[400px] rounded-full bg-[var(--noro-blue)]/10 blur-[100px]" />

    <div class="noro-panel relative z-10 w-full max-w-md border border-[var(--noro-border)] bg-[var(--noro-panel)]/90 p-8 shadow-2xl backdrop-blur-xl md:p-10">
      <div v-if="busy && !appInfo" class="flex flex-col items-center justify-center gap-4 py-12">
        <UIcon name="i-lucide-loader-2" class="size-8 animate-spin text-[var(--noro-magenta)]" />
        <span class="text-sm font-semibold text-[var(--noro-muted)]">Загрузка информации о приложении...</span>
      </div>

      <template v-else>
        <div class="text-center">
          <div class="mx-auto mb-4 flex size-20 items-center justify-center rounded-2xl border-2 border-[var(--noro-magenta)] bg-[var(--noro-magenta)]/10 shadow-[0_0_30px_rgba(232,90,165,0.35)]">
            <img v-if="appInfo?.icon_url" :src="appInfo.icon_url" class="size-full rounded-2xl object-cover" />
            <UIcon v-else name="i-lucide-shield-check" class="size-10 text-[var(--noro-magenta)]" />
          </div>

          <div class="mb-3 inline-flex items-center gap-1.5 rounded-full border border-[var(--noro-magenta)]/30 bg-[var(--noro-magenta)]/10 px-3 py-1 text-[10px] font-black uppercase tracking-wider text-[var(--noro-magenta)]">
            <UIcon name="i-lucide-badge-check" class="size-3.5" />
            Официальное приложение
          </div>

          <h1 class="noro-pixel text-2xl text-[var(--noro-cream)]">
            {{ appInfo?.name || clientId }}
          </h1>
          <p class="mt-2 text-xs leading-relaxed text-[var(--noro-muted)]">
            {{ appInfo?.description || 'Приложение запрашивает доступ к вашему аккаунту Noro Network.' }}
          </p>
        </div>

        <div v-if="auth.user.value" class="my-6 flex items-center gap-3 rounded-xl border border-[var(--noro-border)] bg-[var(--noro-input)] p-3.5">
          <div class="size-10 overflow-hidden rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)]">
            <img :src="`/api/avatar/${auth.user.value.username}`" class="size-full object-cover" onerror="this.src='/icon.png'" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-bold text-[var(--noro-cream)]">{{ auth.user.value.username }}</div>
            <div class="truncate text-xs text-[var(--noro-muted)]">Discord: {{ auth.user.value.discord_username || 'Подключён' }}</div>
          </div>
          <UIcon name="i-lucide-check-circle-2" class="size-5 text-[var(--noro-green)]" />
        </div>

        <div class="mb-6 rounded-xl border border-[var(--noro-border)] bg-[var(--noro-bg)]/60 p-4">
          <div class="mb-2 text-[10px] font-black uppercase tracking-wider text-[var(--noro-muted)]">Запрашиваемые разрешения</div>
          <div class="flex items-center gap-2.5 text-xs font-semibold text-[var(--noro-text)]">
            <UIcon name="i-lucide-user-check" class="size-4 text-[var(--noro-magenta)]" />
            <span>Профиль пользователя и игровой ник ({{ scope }})</span>
          </div>
        </div>

        <UAlert
          v-if="errorMsg"
          color="error"
          variant="subtle"
          icon="i-lucide-triangle-alert"
          :description="errorMsg"
          class="mb-6"
        />

        <div class="grid grid-cols-2 gap-3">
          <button
            type="button"
            class="flex items-center justify-center rounded-xl border border-[var(--noro-border)] bg-[var(--noro-input)] py-3 text-xs font-bold text-[var(--noro-muted)] transition hover:border-[var(--noro-muted)] hover:text-[var(--noro-text)]"
            :disabled="busy"
            @click="onDeny"
          >
            Отклонить
          </button>

          <button
            type="button"
            class="noro-cta flex items-center justify-center gap-2 py-3 text-xs font-bold"
            :disabled="busy"
            @click="onAuthorize"
          >
            <UIcon v-if="busy" name="i-lucide-loader-2" class="size-4 animate-spin" />
            <span v-else>Разрешить доступ</span>
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
