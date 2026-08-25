<script setup lang="ts">
/**
 * Паспорт аккаунта: идентификаторы, даты и флаги.
 *
 * Раньше в карточке был только MC-UUID. Внутренний id спрашивают CLI и запросы
 * в поддержку, даты объясняют половину обращений («он вообще заходил?»), а
 * флаги вроде «не может играть» иначе видно лишь по последствиям.
 */

import type { UserProfile } from '~/types/api'

const props = defineProps<{ user: UserProfile }>()
const { t } = useT()

const when = (iso?: string | null) =>
  iso ? `${new Date(iso).toLocaleDateString()} · ${relativeDateT(iso, t)}` : '—'

/** Показываем только то, что отличается от обычного игрока. */
const flags = computed(() => {
  const u = props.user
  return [
    { on: u.is_root, label: t('admin-users-flag-root'), color: 'text-[var(--noro-cream)]' },
    { on: u.is_local_account, label: t('admin-users-flag-local'), color: 'text-[var(--noro-blue)]' },
    { on: u.can_play === false, label: t('admin-users-flag-no-play'), color: 'text-[var(--noro-danger)]' },
    { on: u.hide_from_online, label: t('admin-users-flag-hidden'), color: 'text-[var(--noro-muted)]' },
    { on: u.silent_join, label: t('admin-users-flag-silent'), color: 'text-[var(--noro-muted)]' },
    { on: u.frozen, label: t('admin-users-flag-frozen'), color: 'text-[var(--noro-danger)]' },
  ].filter(f => f.on)
})
</script>

<template>
  <div class="grid gap-3">
    <div class="rounded-lg bg-[var(--noro-input)] p-3">
      <span class="noro-label mb-1 block">UUID</span>
      <code class="break-all text-xs text-[var(--noro-text)]">{{ user.uuid }}</code>
      <span class="noro-label mb-1 mt-3 block">{{ t('admin-users-account-id') }}</span>
      <code class="break-all text-xs text-[var(--noro-muted)]">{{ user.id }}</code>
    </div>

    <dl class="grid gap-2 rounded-lg bg-[var(--noro-input)] p-3 text-xs">
      <div class="flex flex-wrap items-baseline justify-between gap-2">
        <dt class="text-[var(--noro-muted)]">{{ t('admin-users-registered') }}</dt>
        <dd class="text-[var(--noro-text)]">{{ when(user.created_at) }}</dd>
      </div>
      <div class="flex flex-wrap items-baseline justify-between gap-2">
        <dt class="text-[var(--noro-muted)]">{{ t('admin-users-last-login') }}</dt>
        <dd class="text-[var(--noro-text)]">{{ when(user.last_login_at) }}</dd>
      </div>
    </dl>

    <div v-if="user.banned && user.ban_reason" class="rounded-lg border border-[var(--noro-danger)] bg-[color-mix(in_srgb,var(--noro-danger)_10%,transparent)] p-3">
      <span class="noro-label mb-1 block text-[var(--noro-danger)]">{{ t('admin-users-ban-reason') }}</span>
      <p class="text-xs text-[var(--noro-text)]">{{ user.ban_reason }}</p>
    </div>

    <ul v-if="flags.length" class="flex flex-wrap gap-2">
      <li
        v-for="flag in flags"
        :key="flag.label"
        class="rounded bg-[var(--noro-panel-2)] px-2 py-1 text-[10px] font-black uppercase tracking-wider"
        :class="flag.color"
      >
        {{ flag.label }}
      </li>
    </ul>
  </div>
</template>
