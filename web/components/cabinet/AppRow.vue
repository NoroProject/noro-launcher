<script setup lang="ts">
/** Одно приложение в списке владельца: иконка, состояние, ключи, действия. */

import { APP_STATUS_META, type MyApp } from '~/types/apps'

const props = defineProps<{ app: MyApp; secret: string | null }>()
const emit = defineEmits<{ edit: []; rotate: []; delete: []; icon: [] }>()

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const input = ref<HTMLInputElement | null>(null)
const uploading = ref(false)

const status = computed(() => APP_STATUS_META[props.app.status])

async function copy(text: string) {
  await navigator.clipboard.writeText(text)
  notify.ok()
}

async function uploadIcon(event: Event) {
  const el = event.target as HTMLInputElement
  const file = el.files?.[0]
  if (!file) return
  uploading.value = true
  try {
    const body = new FormData()
    body.append('image', file)
    await auth.request(`/api/me/apps/${props.app.id}/icon`, { method: 'POST', body })
    notify.ok()
    emit('icon')
  } catch (e) {
    notify.fail(e)
  } finally {
    uploading.value = false
    el.value = ''
  }
}
</script>

<template>
  <div class="rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-panel-2)] p-4">
    <div class="flex flex-wrap items-start gap-4">
      <button
        type="button"
        class="group relative grid size-14 shrink-0 place-items-center overflow-hidden rounded-xl border border-[var(--noro-border)] bg-[var(--noro-input)]"
        :title="t('cabinet-myapps-icon-hint')"
        @click="input?.click()"
      >
        <img v-if="app.icon_url" :src="app.icon_url" alt="" class="size-full object-cover">
        <UIcon v-else name="i-lucide-image-plus" class="size-5 text-[var(--noro-muted)]" />
        <span class="absolute inset-0 hidden place-items-center bg-[var(--noro-bg-deep)]/70 group-hover:grid">
          <UIcon :name="uploading ? 'i-lucide-loader-2' : 'i-lucide-upload'" class="size-4 text-[var(--noro-cream)]" :class="uploading && 'animate-spin'" />
        </span>
      </button>
      <input ref="input" type="file" accept="image/png,image/jpeg,image/webp" class="hidden" @change="uploadIcon">

      <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-center gap-2">
          <h3 class="truncate text-sm font-bold text-[var(--noro-cream)]">{{ app.name }}</h3>
          <span class="rounded px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider" :class="status.class">
            {{ t(status.label) }}
          </span>
        </div>
        <p v-if="app.description" class="mt-0.5 text-xs text-[var(--noro-muted)]">{{ app.description }}</p>

        <p v-if="app.review_note" class="mt-2 rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] px-3 py-2 text-xs text-[var(--noro-text)]">
          <span class="font-bold text-[var(--noro-danger)]">{{ t('cabinet-myapps-review-note') }}:</span>
          {{ app.review_note }}
        </p>
      </div>

      <!-- `items-start` обязателен: во флекс-строке кнопки иначе тянутся на всю
           её высоту и превращаются в плитки размером с карточку. -->
      <div class="flex shrink-0 items-start gap-2">
        <AtomButton variant="dark" size="sm" icon="i-lucide-pencil" @click="emit('edit')">
          {{ t('cabinet-myapps-edit') }}
        </AtomButton>
        <AtomButton variant="danger-soft" size="sm" icon="i-lucide-trash-2" @click="emit('delete')" />
      </div>
    </div>

    <!-- Ширина колонки подписей — по самой длинной подписи: фиксированная
         ломала «Адреса возврата» на две строки при пустой половине строки. -->
    <dl class="mt-4 grid grid-cols-[auto_minmax(0,1fr)] items-center gap-x-4 gap-y-2 text-xs">
      <dt class="whitespace-nowrap text-[var(--noro-muted)]">client_id</dt>
      <dd class="flex min-w-0 items-center gap-2">
        <span class="min-w-0 flex-1 truncate font-mono text-[var(--noro-text)]">{{ app.client_id }}</span>
        <AtomButton class="shrink-0" variant="dark" size="sm" icon="i-lucide-copy" @click="copy(app.client_id)" />
      </dd>

      <dt class="whitespace-nowrap text-[var(--noro-muted)]">{{ t('cabinet-myapps-redirects') }}</dt>
      <dd class="min-w-0 truncate font-mono text-[var(--noro-text)]">{{ app.redirect_uris.split('\n').join(', ') }}</dd>

      <dt class="whitespace-nowrap text-[var(--noro-muted)]">{{ t('cabinet-myapps-scopes') }}</dt>
      <dd class="min-w-0 font-mono text-[var(--noro-text)]">{{ app.allowed_scopes.join(' ') }}</dd>
    </dl>

    <!-- Секрет показывается ровно один раз: в базе лежит только его хеш. -->
    <div v-if="secret" class="mt-3 rounded-[var(--noro-r-sm)] border border-[var(--noro-cream)] bg-[color-mix(in_srgb,var(--noro-cream)_8%,transparent)] p-3">
      <div class="noro-label mb-1 text-[var(--noro-cream)]">{{ t('cabinet-myapps-secret-once') }}</div>
      <div class="flex items-center gap-2">
        <code class="min-w-0 flex-1 truncate font-mono text-xs text-[var(--noro-text)]">{{ secret }}</code>
        <AtomButton class="shrink-0" variant="dark" size="sm" icon="i-lucide-copy" @click="copy(secret)" />
      </div>
    </div>

    <button
      v-else
      type="button"
      class="mt-3 text-xs text-[var(--noro-muted)] underline transition hover:text-[var(--noro-text)]"
      @click="emit('rotate')"
    >
      {{ t('cabinet-myapps-rotate') }}
    </button>
  </div>
</template>
