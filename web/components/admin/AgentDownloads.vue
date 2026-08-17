<script setup lang="ts">
import type { AgentFile } from '~/types/agent'

const props = defineProps<{ agents: AgentFile[] }>()
const { t } = useT()

const ICONS: Record<string, string> = {
  paper: 'i-lucide-file-text',
  fabric: 'i-lucide-scissors',
  neoforge: 'i-lucide-hammer',
  forge: 'i-lucide-anvil',
}

/** Группируем по платформе: версий под сотню, плоским списком их не читать. */
const groups = computed(() => {
  const byPlatform = new Map<string, AgentFile[]>()
  for (const agent of props.agents) {
    if (agent.platform === 'wrapper') continue
    const list = byPlatform.get(agent.platform) || []
    list.push(agent)
    byPlatform.set(agent.platform, list)
  }
  return [...byPlatform.entries()]
    .map(([platform, files]) => ({ platform, files: files.sort(byVersion) }))
    .sort((a, b) => a.platform.localeCompare(b.platform))
})

/** Числовое сравнение: иначе 1.21.9 оказывается выше 1.21.10. */
function byVersion(a: AgentFile, b: AgentFile) {
  const parts = (v: string | null) => (v || '').split('.').map(n => Number(n) || 0)
  const [x, y] = [parts(a.mc_version), parts(b.mc_version)]
  for (let i = 0; i < Math.max(x.length, y.length); i++) {
    if ((x[i] || 0) !== (y[i] || 0)) return (x[i] || 0) - (y[i] || 0)
  }
  return 0
}

function sizeKb(bytes: number) {
  return `${Math.round(bytes / 1024)} KB`
}
</script>

<template>
  <section class="noro-panel p-5">
    <div class="mb-4 flex items-center gap-3">
      <div class="grid size-10 place-items-center rounded bg-[var(--noro-green)]/10 text-[var(--noro-green)]">
        <UIcon name="i-lucide-package" class="size-5" />
      </div>
      <div>
        <h3 class="text-lg font-black text-white">{{ t('admin-agent-title') }}</h3>
        <p class="text-sm text-[var(--noro-muted)]">
          {{ t('admin-agent-lead') }}
        </p>
      </div>
    </div>

    <p v-if="!groups.length" class="text-sm text-[var(--noro-amber)]">
      {{ t('admin-agent-not-built') }}
    </p>

    <div v-for="group in groups" :key="group.platform" class="mb-5 last:mb-0">
      <div class="mb-2 flex items-center gap-2">
        <UIcon :name="ICONS[group.platform] || 'i-lucide-box'" class="size-4 text-[var(--noro-muted)]" />
        <h4 class="font-bold text-[var(--noro-text)] capitalize">{{ group.platform }}</h4>
        <span class="text-xs text-[var(--noro-muted)]">{{ t('admin-agent-versions-count', { count: group.files.length }) }}</span>
      </div>
      <div class="flex flex-wrap gap-2">
        <a
          v-for="agent in group.files"
          :key="agent.file"
          :href="agent.url"
          :download="agent.file"
          :title="`${agent.file} — ${sizeKb(agent.size)} — sha1 ${agent.sha1}`"
          class="rounded border border-[var(--noro-border)] bg-black/20 px-3 py-2 font-mono text-xs text-[var(--noro-cream)] transition hover:border-[var(--noro-blue)] hover:text-white"
        >
          {{ agent.mc_version }}
        </a>
      </div>
    </div>
  </section>
</template>
