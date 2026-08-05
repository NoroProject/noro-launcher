<script setup lang="ts">
import type { UserProfile } from '~/types/api'

const auth = useAuth()
await auth.loadMe()

const { data: users, refresh, pending, error } = await useAsyncData('admin-users', () =>
  auth.request<UserProfile[]>('/api/admin/users?limit=200'), { default: () => [] }
)
</script>

<template>
  <NoroShell title="USERS" subtitle="Profiles, bans, roles, and direct permissions">
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

    <section class="noro-panel overflow-hidden">
      <table v-if="users?.length" class="noro-table">
        <thead>
          <tr>
            <th>Player</th>
            <th>Discord</th>
            <th>Roles</th>
            <th>Status</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id">
            <td>
              <div class="font-semibold text-[var(--noro-text)]">{{ user.username }}</div>
              <code class="text-xs text-[var(--noro-muted)]">{{ user.uuid }}</code>
            </td>
            <td>{{ user.discord_username }}</td>
            <td>
              <div class="flex flex-wrap gap-1">
                <UBadge v-for="role in user.roles" :key="role.id" color="neutral" variant="subtle">{{ role.display_name }}</UBadge>
              </div>
            </td>
            <td><UBadge :color="user.banned ? 'error' : 'success'" variant="subtle">{{ user.banned ? 'banned' : 'active' }}</UBadge></td>
            <td class="text-right">
              <AtomButton
                variant="dark"
                icon="i-lucide-settings"
                :to="`/admin/users/${user.id}`"
                class="!min-h-8 !min-w-8 !px-1.5"
              >

              </AtomButton>
            </td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-else icon="i-lucide-users" title="No users yet" />
    </section>
  </NoroShell>
</template>
