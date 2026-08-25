<script setup lang="ts">
/**
 * Свои OAuth2-приложения игрока.
 *
 * Раздел целиком исчезает, когда оператор выключил сторонние приложения: чинить
 * форму, которая всё равно получит отказ, — худший способ сообщить о запрете.
 */

import type { MyApp } from '~/types/apps'

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const apps = ref<MyApp[]>([])
const enabled = ref(true)
const canCreate = ref(false)
const loading = ref(true)
/** Секрет виден один раз — сразу после выпуска, и только в этой вкладке. */
const freshSecret = ref<{ id: string; secret: string } | null>(null)
const editing = ref<MyApp | null>(null)
const creating = ref(false)

async function load() {
  loading.value = true
  try {
    const res = await auth.request<{ items: MyApp[]; enabled: boolean; can_create: boolean }>('/api/me/apps')
    apps.value = res.items
    enabled.value = res.enabled
    canCreate.value = res.can_create
  } catch (e) {
    notify.fail(e)
  } finally {
    loading.value = false
  }
}

function onSaved(app: MyApp, secret?: string) {
  if (secret) freshSecret.value = { id: app.id, secret }
  creating.value = false
  editing.value = null
  load()
}

async function remove(app: MyApp) {
  if (!confirm(t('cabinet-myapps-delete-confirm', { name: app.name }))) return
  try {
    await auth.request(`/api/me/apps/${app.id}`, { method: 'DELETE' })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

async function rotate(app: MyApp) {
  if (!confirm(t('cabinet-myapps-rotate-confirm'))) return
  try {
    const res = await auth.request<{ client_secret: string }>(`/api/me/apps/${app.id}/secret`, { method: 'POST' })
    freshSecret.value = { id: app.id, secret: res.client_secret }
    notify.ok()
  } catch (e) {
    notify.fail(e)
  }
}

onMounted(load)
</script>

<template>
  <NoroCard
    v-if="enabled"
    :title="t('cabinet-myapps-title')"
    :subtitle="t('cabinet-myapps-lead')"
    icon="i-lucide-code"
  >
    <template #actions>
      <AtomButton v-if="canCreate" icon="i-lucide-plus" size="sm" @click="creating = true">
        {{ t('cabinet-myapps-create') }}
      </AtomButton>
    </template>

    <div v-if="loading" class="flex justify-center py-10">
      <UIcon name="i-lucide-loader-2" class="size-6 animate-spin text-[var(--noro-blue)]" />
    </div>

    <div v-else class="grid gap-3">
      <CabinetAppEditor
        v-if="creating"
        @saved="onSaved"
        @cancel="creating = false"
      />

      <template v-for="app in apps" :key="app.id">
        <CabinetAppEditor
          v-if="editing?.id === app.id"
          :app="app"
          @saved="onSaved"
          @cancel="editing = null"
        />
        <CabinetAppRow
          v-else
          :app="app"
          :secret="freshSecret?.id === app.id ? freshSecret.secret : null"
          @edit="editing = app"
          @rotate="rotate(app)"
          @delete="remove(app)"
          @icon="load"
        />
      </template>

      <EmptyState
        v-if="!apps.length && !creating"
        icon="i-lucide-code"
        :title="t('cabinet-myapps-none-title')"
        :text="t('cabinet-myapps-none-text')"
      />
    </div>
  </NoroCard>
</template>
