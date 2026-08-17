<script setup lang="ts">
import type { UserProfile } from '~/types/api'

const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const masterUrl = useRuntimeConfig().public.masterUrl

function headUrl(skinUrl?: string | null) {
  const params = new URLSearchParams({ mode: 'flat-head', scale: '8' })
  if (skinUrl) params.set('url', skinUrl)
  return `${masterUrl}/api/textures/renders?${params.toString()}`
}

const { data: users, refresh, pending, error } = await useAsyncData('admin-users', () =>
  auth.request<UserProfile[]>('/api/admin/users?limit=200'), { default: () => [] }
)
</script>

<template>
  <NoroShell :title="t('nav-admin-users')" :subtitle="t('admin-users-subtitle')">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
    </template>

    <UAlert v-if="error" class="mb-5" color="error" variant="subtle" icon="i-lucide-circle-alert" :description="humanError(error)" />

    <section class="noro-panel overflow-hidden">
      <table v-if="users?.length" class="noro-table">
        <thead>
          <tr>
            <th>{{ t('admin-users-player') }}</th>
            <th>{{ t('admin-users-discord') }}</th>
            <th>{{ t('admin-users-roles') }}</th>
            <th>{{ t('admin-users-status') }}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id">
            <td>
              <div class="flex items-center gap-3">
                <div class="relative size-10 flex-shrink-0">
                  <img
                    :src="user.discord_avatar || '/default-avatar.png'"
                    class="size-10 rounded-lg object-cover border border-[var(--noro-border)] bg-[var(--noro-input)]"
                    alt="Avatar"
                  >
                  <div class="absolute -bottom-1 -right-1 size-5 rounded border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] overflow-hidden shadow">
                    <img
                      :src="headUrl(user.skin_url)"
                      class="size-full object-contain"
                      alt="Skin Head"
                    >
                  </div>
                </div>
                <div>
                  <div class="font-semibold text-[var(--noro-text)]">{{ user.username }}</div>
                  <code class="text-xs text-[var(--noro-muted)]">{{ user.uuid }}</code>
                </div>
              </div>
            </td>
            <td>{{ user.discord_username }}</td>
            <td>
              <div class="flex flex-wrap gap-1">
                <UBadge v-for="role in user.roles" :key="role.id" color="neutral" variant="subtle">{{ role.display_name }}</UBadge>
              </div>
            </td>
            <td><UBadge :color="user.banned ? 'error' : 'success'" variant="subtle">{{ user.banned ? t('admin-users-banned') : t('admin-users-active') }}</UBadge></td>
            <td class="text-right">
              <AtomButton
                variant="dark"
                icon="i-lucide-settings"
                :to="`/admin/users/${user.id}`"
                class="!min-h-8 !min-w-8 !px-1.5"
              />
            </td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-else icon="i-lucide-users" :title="t('admin-users-empty-title')" />
    </section>
  </NoroShell>
</template>
