<script setup lang="ts">
import type { AuditRow } from '~/types/audit'

const props = defineProps<{ row: AuditRow }>()

/** Группа события — по ней красится метка: user.ban → user. */
const group = computed(() => props.row.action.split('.')[0])

const groupColor: Record<string, string> = {
  user: 'var(--noro-magenta)',
  role: 'var(--noro-blue)',
  build: 'var(--noro-cream)',
  server: 'var(--noro-cream)',
  launcher: 'var(--noro-blue)',
  admin_token: 'var(--noro-magenta)',
  storage: 'var(--noro-muted)',
}

const color = computed(() => groupColor[group.value] || 'var(--noro-muted)')

const hasDetails = computed(() =>
  Object.values(props.row.details ?? {}).some((v) => v !== null && v !== undefined && v !== '')
)
</script>

<template>
  <article class="rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4">
    <div class="flex flex-wrap items-center gap-3">
      <span
        class="rounded px-2 py-1 text-[10px] font-black uppercase tracking-wider"
        :style="{ color, background: `color-mix(in srgb, ${color} 16%, transparent)` }"
      >{{ row.action }}</span>

      <span class="text-sm font-bold text-[var(--noro-text)]">{{ row.actor_label }}</span>

      <span v-if="row.target_kind" class="text-xs text-[var(--noro-muted)]">
        → {{ row.target_kind }}
        <NuxtLink
          v-if="row.target_kind === 'user'"
          :to="`/admin/users/${row.target_id}`"
          class="underline hover:text-[var(--noro-cream)]"
        >{{ row.target_id }}</NuxtLink>
        <template v-else>{{ row.target_id }}</template>
      </span>

      <time class="ml-auto text-xs text-[var(--noro-muted)]">
        {{ new Date(row.at).toLocaleString() }}
      </time>
    </div>

    <pre
      v-if="hasDetails"
      class="noro-scroll mt-3 overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs text-[var(--noro-muted)]"
    >{{ JSON.stringify(row.details, null, 2) }}</pre>
  </article>
</template>
