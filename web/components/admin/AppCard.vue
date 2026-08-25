<script setup lang="ts">
/**
 * Приложение в админке: решение по модерации, потолок доступа, охват.
 *
 * Scope'ы правятся тут же галочками, а не отдельной страницей: оператор решает
 * «пускать ли» и «насколько глубоко» одним движением, глядя на одно и то же
 * описание приложения.
 */

import { APP_STATUS_META, type AdminApp, type AppStatus, type ScopeInfo } from '~/types/apps'

const props = defineProps<{ app: AdminApp; scopes: ScopeInfo[]; canManage: boolean }>()
const emit = defineEmits<{ changed: [] }>()

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const busy = ref(false)
const note = ref('')
const asking = ref<AppStatus | null>(null)
const granted = ref<string[]>([...props.app.allowed_scopes])
const iconInput = ref<HTMLInputElement | null>(null)
const uploading = ref(false)

async function uploadIcon(event: Event) {
  const el = event.target as HTMLInputElement
  const file = el.files?.[0]
  if (!file) return
  uploading.value = true
  try {
    const body = new FormData()
    body.append('image', file)
    await auth.request(`/api/admin/oauth-apps/${props.app.id}/icon`, { method: 'POST', body })
    notify.ok()
    emit('changed')
  } catch (e) {
    notify.fail(e)
  } finally {
    uploading.value = false
    el.value = ''
  }
}

const status = computed(() => APP_STATUS_META[props.app.status])
const dirty = computed(
  () => [...granted.value].sort().join() !== [...props.app.allowed_scopes].sort().join(),
)

const scopeText = (s: ScopeInfo) => {
  const key = `oauth-scope-${s.name.replace(':', '-')}`
  const translated = t(key)
  return translated === key ? s.title : translated
}

async function setStatus(next: AppStatus) {
  // Отказ и блокировка требуют причины: её увидит автор в своём кабинете.
  if ((next === 'rejected' || next === 'suspended') && asking.value !== next) {
    asking.value = next
    return
  }
  busy.value = true
  try {
    await auth.request(`/api/admin/oauth-apps/${props.app.id}/status`, {
      method: 'PUT',
      body: { status: next, note: note.value || null },
    })
    notify.ok()
    asking.value = null
    note.value = ''
    emit('changed')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function saveScopes() {
  busy.value = true
  try {
    await auth.request(`/api/admin/oauth-apps/${props.app.id}/scopes`, {
      method: 'PUT',
      body: { scopes: granted.value },
    })
    notify.ok()
    emit('changed')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function remove() {
  if (!confirm(t('admin-apps-delete-confirm', { name: props.app.name }))) return
  busy.value = true
  try {
    await auth.request(`/api/admin/oauth-apps/${props.app.id}`, { method: 'DELETE' })
    notify.ok()
    emit('changed')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="noro-panel p-5">
    <div class="flex flex-wrap items-start gap-4">
      <!-- Иконку заливает и оператор: у официальных приложений владельца нет,
           а на экране согласия наш лаунчер должен выглядеть узнаваемо. -->
      <button
        type="button"
        class="group relative grid size-14 shrink-0 place-items-center overflow-hidden rounded-xl border border-[var(--noro-border)] bg-[var(--noro-input)]"
        :disabled="!canManage"
        :title="t('cabinet-myapps-icon-hint')"
        @click="iconInput?.click()"
      >
        <img v-if="app.icon_url" :src="app.icon_url" alt="" class="size-full object-cover">
        <UIcon v-else name="i-lucide-app-window" class="size-6 text-[var(--noro-muted)]" />
        <span v-if="canManage" class="absolute inset-0 hidden place-items-center bg-[var(--noro-bg-deep)]/70 group-hover:grid">
          <UIcon
            :name="uploading ? 'i-lucide-loader-2' : 'i-lucide-upload'"
            class="size-4 text-[var(--noro-cream)]"
            :class="uploading && 'animate-spin'"
          />
        </span>
      </button>
      <input ref="iconInput" type="file" accept="image/png,image/jpeg,image/webp" class="hidden" @change="uploadIcon">

      <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-center gap-2">
          <h3 class="truncate text-sm font-bold text-[var(--noro-cream)]">{{ app.name }}</h3>
          <span
            v-if="app.is_official"
            class="rounded px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider text-[var(--noro-blue)] bg-[color-mix(in_srgb,var(--noro-blue)_18%,transparent)]"
          >{{ t('oauth-official-app') }}</span>
          <span v-else class="rounded px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider" :class="status.class">
            {{ t(status.label) }}
          </span>
        </div>
        <p v-if="app.description" class="mt-0.5 text-xs text-[var(--noro-muted)]">{{ app.description }}</p>

        <!-- Колонка подписей по самой длинной из них, а не фиксированной
             ширины: под русские «Игроков подключено» её всё равно не угадать,
             и подпись ломалась на две строки при пустой половине карточки. -->
        <dl class="mt-3 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-1 text-xs">
          <dt class="whitespace-nowrap text-[var(--noro-muted)]">{{ t('admin-apps-owner') }}</dt>
          <dd class="min-w-0 truncate text-[var(--noro-text)]">
            <NuxtLink v-if="app.owner" :to="`/admin/users/${app.owner.id}`" class="underline">{{ app.owner.username }}</NuxtLink>
            <span v-else>—</span>
          </dd>

          <dt class="whitespace-nowrap text-[var(--noro-muted)]">client_id</dt>
          <dd class="min-w-0 truncate font-mono text-[var(--noro-text)]">{{ app.client_id }}</dd>

          <dt class="whitespace-nowrap text-[var(--noro-muted)]">{{ t('cabinet-myapps-redirects') }}</dt>
          <dd class="min-w-0 truncate font-mono text-[var(--noro-text)]">{{ app.redirect_uris.join(', ') }}</dd>

          <dt class="whitespace-nowrap text-[var(--noro-muted)]">{{ t('admin-apps-users') }}</dt>
          <dd class="min-w-0 text-[var(--noro-text)]">{{ app.authorized_users }}</dd>
        </dl>
      </div>
    </div>

    <template v-if="canManage && !app.is_official">
      <!-- Потолок доступа. Базовые галочки тоже снимаются: приложению, которому
           хватает ника, незачем и остальное. -->
      <div class="mt-4 border-t border-[var(--noro-border)] pt-4">
        <div class="noro-label mb-2">{{ t('admin-apps-scopes') }}</div>
        <div class="grid gap-1.5 sm:grid-cols-2">
          <label
            v-for="s in scopes"
            :key="s.name"
            class="flex cursor-pointer items-start gap-2 rounded px-2 py-1.5 text-xs transition hover:bg-[var(--noro-panel-2)]"
          >
            <input v-model="granted" type="checkbox" :value="s.name" class="mt-0.5 size-4 accent-[var(--noro-magenta)]">
            <span class="min-w-0">
              <span class="block text-[var(--noro-text)]">{{ scopeText(s) }}</span>
              <span
                class="font-mono text-[10px]"
                :class="s.tier === 'privileged' ? 'text-[var(--noro-magenta)]' : 'text-[var(--noro-muted)]'"
              >{{ s.name }}</span>
            </span>
          </label>
        </div>
        <AtomButton v-if="dirty" class="mt-3" size="sm" icon="i-lucide-save" :loading="busy" @click="saveScopes">
          {{ t('cabinet-save') }}
        </AtomButton>
      </div>

      <div v-if="asking" class="mt-4 grid gap-2">
        <NoroTextarea v-model="note" :rows="2" :placeholder="t('admin-apps-note-placeholder')" />
        <div class="flex gap-2">
          <AtomButton variant="danger" size="sm" :loading="busy" @click="setStatus(asking)">
            {{ t('admin-apps-confirm') }}
          </AtomButton>
          <AtomButton variant="secondary" size="sm" @click="asking = null; note = ''">
            {{ t('cabinet-myapps-cancel') }}
          </AtomButton>
        </div>
      </div>

      <div v-else class="mt-4 flex flex-wrap gap-2">
        <AtomButton
          v-if="app.status !== 'approved'"
          size="sm"
          icon="i-lucide-check"
          :loading="busy"
          @click="setStatus('approved')"
        >{{ t('admin-apps-approve') }}</AtomButton>
        <AtomButton
          v-if="app.status === 'pending'"
          variant="secondary"
          size="sm"
          icon="i-lucide-x"
          @click="setStatus('rejected')"
        >{{ t('admin-apps-reject') }}</AtomButton>
        <AtomButton
          v-if="app.status === 'approved'"
          variant="secondary"
          size="sm"
          icon="i-lucide-ban"
          @click="setStatus('suspended')"
        >{{ t('admin-apps-suspend') }}</AtomButton>
        <AtomButton variant="danger" size="sm" icon="i-lucide-trash-2" :loading="busy" @click="remove">
          {{ t('admin-apps-delete') }}
        </AtomButton>
      </div>

      <p v-if="app.review_note" class="mt-3 text-xs text-[var(--noro-muted)]">
        {{ t('admin-apps-last-note') }}: {{ app.review_note }}
      </p>
    </template>
  </div>
</template>
