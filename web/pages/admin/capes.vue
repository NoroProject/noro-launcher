<script setup lang="ts">
import type { CapeRow } from '~/types/cape'

const auth = useAuth()
await auth.loadMe()

const name = ref('')
const file = ref<File | null>(null)
const busy = ref(false)
const message = ref<string | null>(null)
const { data: capes, refresh, pending, error } = await useAsyncData('admin-capes', () =>
  auth.request<CapeRow[]>('/api/admin/capes'), { default: () => [] }
)

const fileLabel = computed(() => file.value?.name || 'Choose PNG cape')

function onFile(event: Event) {
  const input = event.target as HTMLInputElement
  file.value = input.files?.[0] || null
}

async function uploadCape() {
  if (!file.value || !name.value.trim()) return
  busy.value = true
  message.value = null
  try {
    await auth.upload<CapeRow>('/api/admin/capes', 'cape', file.value, { name: name.value.trim() })
    name.value = ''
    file.value = null
    message.value = 'Cape uploaded'
    await refresh()
  } finally {
    busy.value = false
  }
}

async function deleteCape(cape: CapeRow) {
  busy.value = true
  try {
    await auth.request(`/api/admin/capes/${cape.id}`, { method: 'DELETE' })
    await refresh()
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <NoroShell title="CAPES" subtitle="Upload cape PNG files and assign them from user management">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        Refresh
      </AtomButton>
    </template>

    <UAlert v-if="error" class="mb-5" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="humanError(error)" />
    <UAlert v-if="message" class="mb-5" color="success" variant="subtle" icon="i-lucide-check" :description="message" />

    <div class="grid gap-5 xl:grid-cols-[380px_1fr]">
      <section class="noro-panel p-5">
        <h2 class="text-2xl font-black text-[var(--noro-text)]">Upload Cape</h2>
        <p class="mt-2 text-sm font-medium text-[var(--noro-muted)]">Only admins can upload capes. Players receive capes from their user profile.</p>
        <div class="mt-5 grid gap-3">
          <input v-model="name" class="noro-input" placeholder="Cape name">
          <label class="rounded-lg bg-[var(--noro-input)] p-4">
            <span class="noro-label">PNG file</span>
            <span class="block truncate text-lg font-black text-[var(--noro-cream)]">{{ fileLabel }}</span>
            <input class="mt-3 w-full text-sm text-[var(--noro-text)]" type="file" accept="image/png" @change="onFile">
          </label>
          <button type="button" class="noro-btn noro-btn-primary" :disabled="busy || !file || !name.trim()" @click="uploadCape">
            <UIcon name="i-lucide-upload" class="size-5" />Upload Cape
          </button>
        </div>
      </section>

      <section class="grid gap-4 md:grid-cols-2">
        <article v-for="cape in capes" :key="cape.id" class="group rounded-lg bg-[var(--noro-panel)] p-5 transition-all duration-200 hover:scale-[1.02]">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <h3 class="truncate text-xl font-black text-[var(--noro-cream)]">{{ cape.name }}</h3>
              <p class="mt-1 text-xs font-bold uppercase tracking-wider text-[var(--noro-blue)]">{{ Math.ceil(cape.size / 1024) }} KB</p>
            </div>
            <button class="noro-btn noro-btn-dark !min-h-8 !px-2" :disabled="busy" @click="deleteCape(cape)"><UIcon name="i-lucide-trash-2" class="size-4" /></button>
          </div>
          <div class="mt-4 grid h-32 place-items-center rounded-lg bg-[var(--noro-input)] p-4">
            <img :src="cape.url" :alt="cape.name" class="max-h-full max-w-full object-contain">
          </div>
        </article>
        <EmptyState v-if="!capes?.length" icon="i-lucide-flag" title="No capes uploaded" />
      </section>
    </div>
  </NoroShell>
</template>
