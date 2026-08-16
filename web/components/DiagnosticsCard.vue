<script setup lang="ts">
const props = defineProps<{ userId: string; online: boolean }>()

const auth = useAuth()
const notify = useNotify()

const data = ref<Record<string, any> | null>(null)
const busy = ref(false)

async function load() {
  data.value = await auth.request<Record<string, any> | null>(
    `/api/admin/users/${props.userId}/diagnostics`
  )
}

async function request() {
  busy.value = true
  try {
    await auth.request(`/api/admin/users/${props.userId}/diagnostics`, { method: 'POST' })
    // Лаунчер отвечает по WS — даём ему секунду и перечитываем.
    setTimeout(load, 1500)
    notify.ok('Requested')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function action(name: string) {
  try {
    const res = await auth.request<{ needs_confirmation: boolean }>(
      `/api/admin/users/${props.userId}/action`,
      { method: 'POST', body: { action: name } }
    )
    notify.ok(res.needs_confirmation ? 'The player has to confirm it' : 'Sent')
  } catch (e) {
    notify.fail(e)
  }
}

const rows = computed(() => {
  const d = data.value
  if (!d) return []
  return [
    ['Launcher', d.launcher_version],
    ['OS', `${d.os} ${d.arch}`],
    ['Java', d.java_path || '—'],
    ['Disk free', d.disk_free_mb ? `${d.disk_free_mb} MB` : '—'],
    ['Data size', d.data_size_mb ? `${d.data_size_mb} MB` : '—'],
    ['Master ping', d.master_ping_ms != null ? `${d.master_ping_ms} ms` : 'no answer'],
    ['Last sync error', d.last_sync_error || '—'],
    ['Collected', d.at ? new Date(d.at).toLocaleString() : '—'],
  ]
})

onMounted(() => load())
</script>

<template>
  <section class="noro-panel p-5 space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-black text-[var(--noro-text)]">Diagnostics</h3>
      <AtomButton variant="dark" icon="i-lucide-activity" :loading="busy" :disabled="!online" @click="request">
        Collect
      </AtomButton>
    </div>

    <p class="text-xs text-[var(--noro-muted)]">
      Versions, hardware and link speed — nothing personal, so the player is not
      asked. Closes half of the "it won't start" tickets without a single log file.
    </p>

    <dl v-if="data" class="grid gap-1 text-xs">
      <div v-for="[k, v] in rows" :key="k" class="flex justify-between gap-4">
        <dt class="text-[var(--noro-muted)]">{{ k }}</dt>
        <dd class="truncate text-right text-[var(--noro-text)]">{{ v }}</dd>
      </div>
    </dl>
    <p v-else class="text-xs text-[var(--noro-muted)]">Nothing collected yet.</p>

    <div class="flex flex-wrap gap-2">
      <AtomButton variant="dark" class="!min-h-8" :disabled="!online" @click="action('verify_integrity')">
        Verify files
      </AtomButton>
      <AtomButton variant="dark" class="!min-h-8" :disabled="!online" @click="action('clear_asset_cache')">
        Clear assets
      </AtomButton>
      <AtomButton variant="dark" class="!min-h-8" :disabled="!online" @click="action('restart_launcher')">
        Restart launcher
      </AtomButton>
    </div>
  </section>
</template>
