<script setup lang="ts">
/**
 * Форма приложения: название, описание, адреса возврата.
 *
 * Правка возвращает приложение на модерацию — об этом сказано прямо в форме, а
 * не выясняется после сохранения по сменившемуся бейджу.
 */

import type { MyApp, ScopeInfo } from '~/types/apps'

const props = defineProps<{ app?: MyApp }>()
const emit = defineEmits<{ saved: [app: MyApp, secret?: string]; cancel: [] }>()

const auth = useAuth()
const api = useApi()
const notify = useNotify()
const { t } = useT()

const form = reactive({
  name: props.app?.name ?? '',
  description: props.app?.description ?? '',
  redirect_uris: props.app?.redirect_uris ?? '',
  // Пустой список у нового приложения: просить всё подряд не нужно никому, а
  // лишний доступ в списке на экране согласия отпугивает игроков.
  scopes: [...(props.app?.allowed_scopes ?? [])],
})
const busy = ref(false)

/**
 * Выбирать можно только базовые доступы.
 *
 * Привилегированные выдаёт оператор поимённо — показывать их здесь галочками
 * значило бы предлагать то, чего форма всё равно не сохранит.
 */
const { data: scopes } = await useAsyncData('oauth-basic-scopes', async () => {
  try {
    const all = await api.request<ScopeInfo[]>('/api/oauth2/scopes')
    return all.filter(s => s.tier === 'basic')
  } catch {
    return [] as ScopeInfo[]
  }
}, { default: () => [] as ScopeInfo[] })

const scopeText = (s: ScopeInfo) => {
  const key = `oauth-scope-${s.name.replace(':', '-')}`
  const translated = t(key)
  return translated === key ? s.title : translated
}

async function save() {
  busy.value = true
  try {
    const body = {
      name: form.name,
      description: form.description || null,
      redirect_uris: form.redirect_uris,
      scopes: form.scopes,
    }
    if (props.app) {
      const app = await auth.request<MyApp>(`/api/me/apps/${props.app.id}`, { method: 'PUT', body })
      notify.ok()
      emit('saved', app)
    } else {
      const app = await auth.request<MyApp & { client_secret: string }>('/api/me/apps', {
        method: 'POST',
        body,
      })
      notify.ok()
      emit('saved', app, app.client_secret)
    }
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="grid gap-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-cream)] bg-[var(--noro-panel-2)] p-4">
    <label class="block">
      <span class="noro-label mb-1.5 block">{{ t('cabinet-myapps-name') }}</span>
      <input v-model="form.name" class="noro-input w-full" maxlength="100" :placeholder="t('cabinet-myapps-name-hint')">
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">{{ t('cabinet-myapps-desc') }}</span>
      <input v-model="form.description" class="noro-input w-full" maxlength="500" :placeholder="t('cabinet-myapps-desc-hint')">
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">{{ t('cabinet-myapps-redirects') }}</span>
      <NoroTextarea v-model="form.redirect_uris" :rows="3" placeholder="https://example.com/callback" />
      <span class="mt-1 block text-xs text-[var(--noro-muted)]">{{ t('cabinet-myapps-redirects-hint') }}</span>
    </label>

    <div>
      <span class="noro-label mb-1.5 block">{{ t('cabinet-myapps-pick-scopes') }}</span>
      <div class="grid gap-1.5 sm:grid-cols-2">
        <label
          v-for="s in scopes"
          :key="s.name"
          class="flex cursor-pointer items-start gap-2 rounded px-2 py-1.5 text-xs transition hover:bg-[var(--noro-panel)]"
        >
          <input v-model="form.scopes" type="checkbox" :value="s.name" class="mt-0.5 size-4 accent-[var(--noro-magenta)]">
          <span class="min-w-0">
            <span class="block text-[var(--noro-text)]">{{ scopeText(s) }}</span>
            <span class="font-mono text-[10px] text-[var(--noro-muted)]">{{ s.name }}</span>
          </span>
        </label>
      </div>
      <span class="mt-1 block text-xs text-[var(--noro-muted)]">{{ t('cabinet-myapps-pick-scopes-hint') }}</span>
    </div>

    <p v-if="app" class="text-xs text-[var(--noro-muted)]">{{ t('cabinet-myapps-edit-requeues') }}</p>

    <div class="flex gap-2">
      <AtomButton icon="i-lucide-save" :loading="busy" @click="save">{{ t('cabinet-save') }}</AtomButton>
      <AtomButton variant="secondary" :disabled="busy" @click="emit('cancel')">{{ t('cabinet-myapps-cancel') }}</AtomButton>
    </div>
  </div>
</template>
