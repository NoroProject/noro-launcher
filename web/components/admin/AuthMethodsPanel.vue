<script setup lang="ts">
/**
 * Способы входа: платформы OAuth и passkey.
 *
 * Своя ручка и своё сохранение — по кнопке у каждой платформы, а не общей
 * кнопкой страницы: у метода свои ключи и свой тумблер, и «сохранить всё»
 * тут означало бы перезапись секретов, которых страница не видела.
 */

interface AuthMethodItem {
  method: string
  name: string
  kind: 'oauth' | 'passkey'
  enabled: boolean
  configured: boolean
  client_id?: string
  secret_set?: boolean
  redirect_uri?: string
  from_env?: boolean
}

const auth = useAuth()
const notify = useNotify()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)

const items = ref<AuthMethodItem[]>([])
const pending = ref(false)
/** Введённые секреты. Пустая строка — «не трогать»: текущий не показывается. */
const secrets = ref<Record<string, string>>({})

async function load() {
  pending.value = true
  try {
    items.value = (await auth.request<{ methods: AuthMethodItem[] }>('/api/admin/auth-methods')).methods
  } catch (e) {
    notify.fail(e, 'Failed to load sign-in methods')
  } finally {
    pending.value = false
  }
}

async function save(item: AuthMethodItem) {
  pending.value = true
  try {
    await auth.request(`/api/admin/auth-methods/${item.method}`, {
      method: 'PUT',
      body: {
        client_id: item.client_id ?? '',
        client_secret: secrets.value[item.method] || null,
        enabled: item.enabled,
      },
    })
    secrets.value[item.method] = ''
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
    await load()
  } finally {
    pending.value = false
  }
}

async function copy(text: string) {
  await navigator.clipboard.writeText(text)
  notify.ok()
}

onMounted(() => load())
</script>

<template>
  <div class="grid gap-4">
    <NoroCard
      v-for="item in items"
      :key="item.method"
      :title="item.name"
      :subtitle="item.kind === 'oauth' ? t('admin-auth-oauth-lead') : t('admin-auth-passkey-lead')"
      :icon="item.kind === 'oauth' ? 'i-lucide-log-in' : 'i-lucide-key-round'"
    >
      <template #actions>
        <label class="flex cursor-pointer items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--noro-muted)]">
          {{ item.enabled ? t('admin-auth-on') : t('admin-auth-off') }}
          <input
            v-model="item.enabled"
            type="checkbox"
            class="size-5 cursor-pointer accent-[var(--noro-magenta)]"
            :disabled="!can('noro.admin.settings.edit')"
            @change="save(item)"
          >
        </label>
      </template>

      <div class="grid gap-3">
        <UAlert
          v-if="item.enabled && !item.configured"
          color="warning"
          variant="subtle"
          icon="i-lucide-triangle-alert"
          :description="item.kind === 'oauth' ? t('admin-auth-not-configured') : t('admin-auth-passkey-blocked')"
        />

        <template v-if="item.kind === 'oauth'">
          <label class="block">
            <span class="noro-label mb-1.5 block">Client ID</span>
            <input
              v-model="item.client_id"
              class="noro-input w-full font-mono"
              :disabled="item.from_env || !can('noro.admin.settings.edit')"
            >
          </label>

          <label class="block">
            <span class="noro-label mb-1.5 block">
              Client secret
              <span class="ml-2 font-normal normal-case text-[var(--noro-muted)]">
                {{ item.secret_set ? t('admin-auth-secret-set') : t('admin-auth-secret-unset') }}
              </span>
            </span>
            <input
              v-model="secrets[item.method]"
              type="password"
              autocomplete="new-password"
              class="noro-input w-full font-mono"
              :placeholder="t('admin-auth-secret-placeholder')"
              :disabled="!can('noro.admin.settings.edit')"
            >
          </label>

          <div>
            <span class="noro-label mb-1.5 block">{{ t('admin-auth-redirect') }}</span>
            <div class="flex items-center gap-2">
              <code class="noro-scroll min-w-0 flex-1 overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs">{{ item.redirect_uri }}</code>
              <AtomButton variant="dark" size="sm" icon="i-lucide-copy" @click="copy(item.redirect_uri || '')" />
            </div>
          </div>

          <div v-if="can('noro.admin.settings.edit')">
            <AtomButton icon="i-lucide-save" :loading="pending" @click="save(item)">{{ t('cabinet-save') }}</AtomButton>
          </div>
        </template>
      </div>
    </NoroCard>
  </div>
</template>
