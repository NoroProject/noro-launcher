<script setup lang="ts">
/**
 * Шаг визарда: чем игроки будут входить.
 *
 * Платформ несколько, и настроить достаточно любую одну — остальные оператор
 * добавит потом в админке. Ключи уезжают в `auth_methods`, а не в настройки
 * инстанса: там же их потом и правят.
 */

const props = defineProps<{ apiUrl: string }>()
const emit = defineEmits<{ next: [] }>()

const auth = useApi()
const notify = useNotify()

const PROVIDERS = [
  { id: 'discord', name: 'Discord' },
  { id: 'twitch', name: 'Twitch' },
  { id: 'google', name: 'Google' },
]

const active = ref('discord')
const form = ref<Record<string, { client_id: string; client_secret: string }>>(
  Object.fromEntries(PROVIDERS.map((p) => [p.id, { client_id: '', client_secret: '' }]))
)
const saved = ref<string[]>([])
const busy = ref(false)

const redirectUri = (id: string) => `${props.apiUrl.replace(/\/$/, '')}/auth/${id}/callback`

async function save(id: string) {
  busy.value = true
  try {
    await auth.request('/api/setup/sign-in', {
      method: 'POST',
      body: {
        method: id,
        client_id: form.value[id].client_id.trim(),
        client_secret: form.value[id].client_secret.trim() || null,
      },
    })
    if (!saved.value.includes(id)) saved.value.push(id)
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="grid gap-4">
    <div class="flex flex-wrap gap-2">
      <button
        v-for="p in PROVIDERS"
        :key="p.id"
        type="button"
        class="rounded px-3 py-1.5 text-xs font-black uppercase tracking-wider transition"
        :class="active === p.id
          ? 'bg-[var(--noro-magenta)] text-[var(--noro-white)]'
          : 'bg-[var(--noro-input)] text-[var(--noro-muted)] hover:text-[var(--noro-cream)]'"
        @click="active = p.id"
      >
        {{ p.name }}
        <span v-if="saved.includes(p.id)">✓</span>
      </button>
    </div>

    <label class="block">
      <span class="noro-label mb-1.5 block">Client ID</span>
      <input v-model="form[active].client_id" class="noro-input w-full font-mono" placeholder="1512048650258219089">
    </label>

    <label class="block">
      <span class="noro-label mb-1.5 block">Client secret</span>
      <input v-model="form[active].client_secret" type="password" autocomplete="new-password" class="noro-input w-full font-mono">
    </label>

    <div>
      <span class="noro-label mb-1.5 block">Redirect URI for the provider portal</span>
      <pre class="noro-scroll overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs">{{ redirectUri(active) }}</pre>
    </div>

    <div class="flex flex-wrap gap-2">
      <AtomButton variant="secondary" icon="i-lucide-save" :loading="busy" @click="save(active)">Save provider</AtomButton>
      <AtomButton icon="i-lucide-arrow-right" @click="emit('next')">Continue</AtomButton>
    </div>

    <p class="text-xs text-[var(--noro-muted)]">
      One provider is enough to start. The rest can be added later in Admin → Sign-in.
    </p>
  </div>
</template>
