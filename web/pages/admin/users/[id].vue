<script setup lang="ts">
import type { Role, ServerRow, UserProfile } from '~/types/api'
import type { CapeRow } from '~/types/cape'
import type { PermissionEntry } from '~/types/permissions'
import { toPermissionEntries } from '~/types/permissions'

const route = useRoute()
const auth = useAuth()
await auth.loadMe()

const id = computed(() => String(route.params.id))
const { data: user, refresh: refreshUser } = await useAsyncData(`admin-user-${id.value}`, () =>
  auth.request<UserProfile>(`/api/admin/users/${id.value}`)
)
const { data: roles } = await useAsyncData('admin-user-roles-list', () =>
  auth.request<Role[]>('/api/admin/roles'), { default: () => [] }
)
const { data: capes } = await useAsyncData('admin-user-capes-list', () =>
  auth.request<CapeRow[]>('/api/admin/capes'), { default: () => [] }
)
const { data: servers } = await useAsyncData('admin-user-servers', () =>
  auth.request<ServerRow[]>('/api/admin/servers'), { default: () => [] }
)

const permissions = computed(() =>
  toPermissionEntries(user.value?.permission_grants, user.value?.permissions || [])
)
const grants = usePermissionGrants(computed(() => `/api/admin/users/${id.value}`))

const selectedCape = ref('')
const banReason = ref('')
const busy = ref<string | null>(null)

const currentCape = computed(() => capes.value.find(cape => cape.url === user.value?.cape_url))

watch([user, capes], () => {
  selectedCape.value = currentCape.value?.id || ''
}, { immediate: true })

async function run(name: string, action: () => Promise<void>) {
  busy.value = name
  try {
    await action()
    await refreshUser()
  } finally {
    busy.value = null
  }
}

async function setBan(banned: boolean) {
  await run('ban', () => auth.request(`/api/admin/users/${id.value}/ban`, {
    method: 'PUT',
    body: { banned, reason: banned ? banReason.value || null : null }
  }))
}

async function assignCape() {
  await run('cape', () => auth.request(`/api/admin/users/${id.value}/cape`, {
    method: 'PUT',
    body: { cape_id: selectedCape.value || null }
  }))
}

async function addRole(roleId: string) {
  await run('role', () => auth.request(`/api/admin/users/${id.value}/roles/${roleId}`, { method: 'POST' }))
}

async function removeRole(roleId: string) {
  await run(`role-${roleId}`, () => auth.request(`/api/admin/users/${id.value}/roles/${roleId}`, { method: 'DELETE' }))
}

async function addPermission(entries: PermissionEntry[]) {
  await run('perm', () => grants.addMany(entries))
}

async function removePermission(entry: PermissionEntry) {
  await run(`perm-${entry.permission}`, () => grants.remove(entry))
}

async function movePermission(entry: PermissionEntry, serverId: string | null) {
  await run(`perm-${entry.permission}`, () => grants.move(entry, serverId))
}
</script>

<template>
  <NoroShell :title="user?.username || 'USER'" :subtitle="user?.discord_username">
    <template #actions>
      <AtomButton variant="dark" icon="i-lucide-arrow-left" to="/admin/users">Back</AtomButton>
    </template>

    <EmptyState v-if="!user" icon="i-lucide-search-x" title="User not found" />

    <div v-else class="grid gap-5 xl:grid-cols-[380px_1fr]">
      <section class="grid gap-5">
        <div class="noro-panel p-5">
          <div class="flex items-center gap-4">
            <img v-if="user.discord_avatar" :src="user.discord_avatar" alt="" class="size-16 rounded-lg object-cover">
            <div v-else class="grid size-16 place-items-center rounded-lg bg-[var(--noro-magenta)] text-2xl font-black text-[var(--noro-white)]">{{ user.username.slice(0, 1).toUpperCase() }}</div>
            <div class="min-w-0">
              <h2 class="truncate text-xl font-black text-[var(--noro-text)]">{{ user.username }}</h2>
              <p class="truncate text-sm text-[var(--noro-muted)]">{{ user.discord_username }}</p>
              <UBadge class="mt-2" :color="user.banned ? 'error' : 'success'" variant="subtle">{{ user.banned ? 'banned' : 'active' }}</UBadge>
            </div>
          </div>
          <div class="mt-4 rounded-lg bg-[var(--noro-input)] p-3">
            <code class="break-all text-xs text-[var(--noro-muted)]">{{ user.uuid }}</code>
          </div>
        </div>
        <SkinPreview3D :skin-url="user.skin_url" :cape-url="user.cape_url" />
      </section>

      <section class="grid gap-5">
        <div class="grid gap-5 lg:grid-cols-2">
          <div class="noro-panel p-5">
            <h2 class="mb-4 text-xl font-black text-[var(--noro-text)]">Cape Assignment</h2>
            <select v-model="selectedCape" class="noro-input">
              <option value="">No cape</option>
              <option v-for="cape in capes" :key="cape.id" :value="cape.id">{{ cape.name }}</option>
            </select>
            <div class="mt-4 flex items-center justify-between gap-3 rounded-lg bg-[var(--noro-input)] p-3">
              <span class="truncate text-sm font-bold text-[var(--noro-muted)]">{{ currentCape?.name || 'No cape assigned' }}</span>
              <AtomButton
                variant="primary"
                icon="i-lucide-save"
                :disabled="busy === 'cape'"
                @click="assignCape"
              >
                Apply
              </AtomButton>
            </div>
          </div>

          <div class="noro-panel p-5">
            <h2 class="mb-4 text-xl font-black text-[var(--noro-text)]">Moderation</h2>
            <input v-model="banReason" class="noro-input" placeholder="Ban reason">
            <div class="mt-4 flex flex-wrap gap-2">
              <AtomButton variant="danger" :loading="busy === 'ban'" icon="i-lucide-ban" @click="setBan(true)">Ban</AtomButton>
              <AtomButton variant="secondary" :loading="busy === 'ban'" icon="i-lucide-check" @click="setBan(false)">Unban</AtomButton>
            </div>
          </div>
        </div>

        <AdminUserRoles :roles="roles" :user-roles="user.roles" :busy="busy" @add="addRole" @remove="removeRole" />
        <AdminPermissionEditor
          title="Direct Permissions"
          subtitle="Granted to this player on top of their roles. Pick a build to scope a permission to it."
          :entries="permissions"
          :servers="servers"
          :busy="busy"
          @add="addPermission"
          @remove="removePermission"
          @move="movePermission"
        />
      </section>
    </div>
  </NoroShell>
</template>
