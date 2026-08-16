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
const { data: userCapesData, refresh: refreshUserCapes } = await useAsyncData(`admin-user-capes-access-${id.value}`, () =>
  auth.request<{ granted_cape_ids: string[] }>(`/api/admin/users/${id.value}/capes`), { default: () => ({ granted_cape_ids: [] }) }
)

interface SkinPresetItem {
  id: string
  name: string
  skin_url: string
}

const { data: skinPresets, refresh: refreshSkinPresets } = await useAsyncData(`admin-user-skin-presets-${id.value}`, () =>
  auth.request<SkinPresetItem[]>(`/api/admin/users/${id.value}/skin-presets`), { default: () => [] }
)

const showImpersonate = ref(false)
const showRequestLogs = ref(false)
const launcherOnline = ref(false)
const activeTab = ref<'profile' | 'skin_capes' | 'moderation'>('profile')

const permissions = computed(() =>
  toPermissionEntries(user.value?.permission_grants, user.value?.permissions || [])
)
const grants = usePermissionGrants(computed(() => `/api/admin/users/${id.value}`))

const selectedActiveCape = ref('')
const capeDropdownOpen = ref(false)
const currentSelectedCapeObj = computed(() => capes.value.find(c => c.id === selectedActiveCape.value))
const grantedCapeIds = ref<Set<string>>(new Set())
const banReason = ref('')
const busy = ref<string | null>(null)
const skinFile = ref<File | null>(null)

const currentCape = computed(() => capes.value.find(cape => cape.url === user.value?.cape_url))

watch([user, capes, userCapesData], () => {
  selectedActiveCape.value = currentCape.value?.id || ''
  if (userCapesData.value?.granted_cape_ids) {
    grantedCapeIds.value = new Set(userCapesData.value.granted_cape_ids)
  }
}, { immediate: true })

async function run(name: string, action: () => Promise<void>) {
  busy.value = name
  try {
    await action()
    await refreshUser()
    await refreshUserCapes()
    await refreshSkinPresets()
  } finally {
    busy.value = null
  }
}

async function selectPresetForUser(skinUrl: string) {
  await run('select-preset', async () => {
    await auth.request(`/api/admin/users/${id.value}/skin-presets/select`, {
      method: 'POST',
      body: { skin_url: skinUrl }
    })
  })
}

async function deletePresetForUser(presetId: string) {
  await run(`delete-preset-${presetId}`, async () => {
    await auth.request(`/api/admin/users/${id.value}/skin-presets/${presetId}`, {
      method: 'DELETE'
    })
  })
}

async function setBan(banned: boolean) {
  await run('ban', () => auth.request(`/api/admin/users/${id.value}/ban`, {
    method: 'PUT',
    body: { banned, reason: banned ? banReason.value || null : null }
  }))
}

function toggleCapeGrant(capeId: string) {
  const next = new Set(grantedCapeIds.value)
  if (next.has(capeId)) {
    next.delete(capeId)
  } else {
    next.add(capeId)
  }
  grantedCapeIds.value = next
}

async function saveCapesAccess() {
  await run('save-capes', () => auth.request(`/api/admin/users/${id.value}/capes`, {
    method: 'PUT',
    body: {
      granted_cape_ids: Array.from(grantedCapeIds.value),
      active_cape_id: selectedActiveCape.value || null
    }
  }))
}

async function addRole(roleId: string) {
  await run('role', () => auth.request(`/api/admin/users/${id.value}/roles/${roleId}`, { method: 'POST' }))
}

async function removeRole(roleId: string) {
  await run(`role-${roleId}`, () => auth.request(`/api/admin/users/${id.value}/roles/${roleId}`, { method: 'DELETE' }))
}

async function addPermission(entries: PermissionEntry[]) {
  await run(entries.length === 1 ? `perm-${entries[0]!.permission}` : 'perm', () =>
    grants.addMany(entries)
  )
}

async function removePermission(entries: PermissionEntry[]) {
  await run(`perm-${entries[0]?.permission}`, () => grants.removeMany(entries))
}

async function uploadSkinForUser(file: File) {
  await run('upload-skin', async () => {
    const fd = new FormData()
    fd.append('skin', file)
    await auth.request(`/api/admin/users/${id.value}/skin`, {
      method: 'POST',
      body: fd
    })
  })
}

async function removeSkinForUser() {
  await run('remove-skin', () => auth.request(`/api/admin/users/${id.value}/skin`, {
    method: 'DELETE'
  }))
}

const skinInput = ref<HTMLInputElement | null>(null)

function triggerSkinPicker() {
  skinInput.value?.click()
}

function onSkinFilePicked(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files?.[0]) {
    uploadSkinForUser(input.files[0])
  }
}
</script>

<template>
  <NoroShell :title="user?.username || 'USER'" :subtitle="user?.discord_username">
    <template #actions>
      <AtomButton variant="dark" icon="i-lucide-arrow-left" to="/admin/users">Back</AtomButton>
    </template>

    <EmptyState v-if="!user" icon="i-lucide-search-x" title="User not found" />

    <div v-else class="space-y-6">
      <!-- Tabs Navigation -->
      <div class="noro-panel flex gap-2 p-2">
        <button
          class="flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-bold transition"
          :class="activeTab === 'profile' ? 'bg-[var(--noro-input)] text-[var(--noro-cream)] shadow' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
          @click="activeTab = 'profile'"
        >
          <UIcon name="i-lucide-user" class="size-4" />
          <span>Profile & Permissions</span>
        </button>

        <button
          class="flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-bold transition"
          :class="activeTab === 'skin_capes' ? 'bg-[var(--noro-input)] text-[var(--noro-cream)] shadow' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
          @click="activeTab = 'skin_capes'"
        >
          <UIcon name="i-lucide-sparkles" class="size-4" />
          <span>Skin & Capes Access</span>
          <UBadge color="primary" variant="subtle" class="ml-1">{{ userCapesData.granted_cape_ids.length }} capes</UBadge>
        </button>

        <button
          class="flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-bold transition"
          :class="activeTab === 'moderation' ? 'bg-[var(--noro-input)] text-[var(--noro-cream)] shadow' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
          @click="activeTab = 'moderation'"
        >
          <UIcon name="i-lucide-shield-alert" class="size-4" />
          <span>Moderation</span>
        </button>
      </div>

      <!-- Tab 1: Profile & Permissions -->
      <div v-if="activeTab === 'profile'" class="grid gap-5 xl:grid-cols-[360px_1fr]">
        <div class="space-y-5">
        <UserLauncherPanel v-model:online="launcherOnline" :user-id="id" @impersonate="showImpersonate = true" @request-logs="showRequestLogs = true" />
        <DiagnosticsCard :user-id="id" :online="launcherOnline" />
        <div class="noro-panel h-fit p-5 space-y-4">
          <div class="flex items-center gap-4">
            <img v-if="user.discord_avatar" :src="user.discord_avatar" alt="" class="size-16 rounded-lg object-cover">
            <div v-else class="grid size-16 place-items-center rounded-lg bg-[var(--noro-magenta)] text-2xl font-black text-[var(--noro-white)]">{{ user.username.slice(0, 1).toUpperCase() }}</div>
            <div class="min-w-0">
              <h2 class="truncate text-xl font-black text-[var(--noro-text)]">{{ user.username }}</h2>
              <p class="truncate text-sm text-[var(--noro-muted)]">{{ user.discord_username }}</p>
              <UBadge class="mt-2" :color="user.banned ? 'error' : 'success'" variant="subtle">{{ user.banned ? 'banned' : 'active' }}</UBadge>
            </div>
          </div>
          <div class="rounded-lg bg-[var(--noro-input)] p-3">
            <span class="text-xs uppercase tracking-wider text-[var(--noro-muted)] font-bold block mb-1">UUID</span>
            <code class="break-all text-xs text-[var(--noro-text)]">{{ user.uuid }}</code>
          </div>
        </div>

        <div class="grid gap-5">
          <AdminUserRoles :roles="roles" :user-roles="user.roles" :busy="busy" @add="addRole" @remove="removeRole" />
          <AdminPermissionEditor
            title="Direct Permissions"
            subtitle="Granted to this player on top of their roles. Pick the builds a permission applies to, or all of them."
            :entries="permissions"
            :servers="servers"
            :busy="busy"
            @add="addPermission"
            @remove="removePermission"
          />
        </div>
        </div>
      </div>

      <!-- Tab 2: Skin & Capes Access -->
      <div v-if="activeTab === 'skin_capes'" class="grid gap-5 xl:grid-cols-[380px_1fr]">
        <!-- 3D Skin & Controls Panel -->
        <section class="space-y-4">
          <div class="noro-panel p-5 space-y-4">
            <h3 class="noro-label">3D Skin Preview</h3>
            <SkinPreview3D :skin-url="user.skin_url" :cape-url="user.cape_url" />
            
            <!-- Panel under skin -->
            <div class="rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] p-4 space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-[var(--noro-muted)] uppercase">Custom Skin</span>
                <UBadge :color="user.skin_url ? 'primary' : 'neutral'" variant="subtle">{{ user.skin_url ? 'Uploaded' : 'Default' }}</UBadge>
              </div>

              <div class="flex items-center gap-2">
                <input ref="skinInput" type="file" accept="image/png" class="hidden" @change="onSkinFilePicked">
                <AtomButton
                  variant="primary"
                  icon="i-lucide-upload"
                  :disabled="busy === 'upload-skin'"
                  @click="triggerSkinPicker"
                >
                  Upload Skin
                </AtomButton>

                <AtomButton
                  v-if="user.skin_url"
                  variant="danger"
                  icon="i-lucide-trash-2"
                  :disabled="busy === 'remove-skin'"
                  @click="removeSkinForUser"
                >
                  Reset Skin
                </AtomButton>
              </div>
            </div>

            <!-- Saved Skin Presets Panel -->
            <div class="noro-panel p-5 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="noro-label">Skin Presets</h3>
                  <p class="text-xs text-[var(--noro-muted)]">Saved player skins gallery</p>
                </div>
                <UBadge color="primary" variant="subtle">{{ skinPresets.length }} presets</UBadge>
              </div>

              <div v-if="skinPresets.length" class="grid grid-cols-2 gap-3 max-h-80 overflow-y-auto pr-1 noro-scroll">
                <div
                  v-for="preset in skinPresets"
                  :key="preset.id"
                  class="group relative flex flex-col justify-between rounded-lg border p-3 transition"
                  :class="user.skin_url === preset.skin_url ? 'border-[var(--noro-cream)] bg-[var(--noro-input)] shadow-md' : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] hover:border-[var(--noro-muted)]'"
                >
                  <div class="flex items-center justify-between gap-1 mb-2">
                    <span class="truncate text-xs font-bold text-[var(--noro-text)]" :title="preset.name">{{ preset.name }}</span>
                    <button
                      type="button"
                      class="text-[var(--noro-muted)] hover:text-red-400 p-0.5 rounded transition opacity-0 group-hover:opacity-100"
                      title="Delete preset"
                      @click.stop="deletePresetForUser(preset.id)"
                    >
                      <UIcon name="i-lucide-trash-2" class="size-3.5" />
                    </button>
                  </div>

                  <div class="flex items-center justify-center py-2 bg-[var(--noro-input)] rounded border border-[var(--noro-border)] mb-2 overflow-hidden">
                    <img :src="preset.skin_url" alt="" class="h-16 w-auto object-contain image-render-pixelated">
                  </div>

                  <div>
                    <UBadge v-if="user.skin_url === preset.skin_url" color="primary" variant="subtle" class="w-full justify-center text-[10px]">
                      Active
                    </UBadge>
                    <AtomButton
                      v-else
                      variant="secondary"
                      size="sm"
                      class="w-full justify-center text-[10px]"
                      :disabled="busy === 'select-preset'"
                      @click="selectPresetForUser(preset.skin_url)"
                    >
                      Equip
                    </AtomButton>
                  </div>
                </div>
              </div>
              <EmptyState v-else icon="i-lucide-images" title="No skin presets" text="Player hasn't saved any presets yet." />
            </div>
          </div>
        </section>

        <!-- Capes Management & Access Permissions -->
        <section class="space-y-5">
          <!-- Active Cape Selection -->
          <div class="noro-panel p-5 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-lg font-bold text-[var(--noro-text)]">Active Cape</h2>
                <p class="text-xs text-[var(--noro-muted)]">Select which granted cape is equipped on player's model</p>
              </div>
              <AtomButton
                variant="primary"
                icon="i-lucide-save"
                :disabled="busy === 'save-capes'"
                @click="saveCapesAccess"
              >
                Save Changes
              </AtomButton>
            </div>

            <!-- Custom Cape Select Dropdown -->
            <div class="relative">
              <button
                type="button"
                class="flex w-full items-center justify-between rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] p-3 text-left transition hover:border-[var(--noro-cream)] focus:outline-none"
                @click="capeDropdownOpen = !capeDropdownOpen"
              >
                <div class="flex items-center gap-3">
                  <div class="grid size-10 place-items-center rounded bg-[var(--noro-bg-deep)] border border-[var(--noro-border)]">
                    <CapePreview v-if="selectedActiveCape && currentSelectedCapeObj" :url="currentSelectedCapeObj.url" class="h-8 shadow" />
                    <UIcon v-else name="i-lucide-eye-off" class="size-5 text-[var(--noro-muted)]" />
                  </div>
                  <div>
                    <div class="font-bold text-sm text-[var(--noro-text)]">
                      {{ selectedActiveCape && currentSelectedCapeObj ? currentSelectedCapeObj.name : 'No cape (Disabled)' }}
                    </div>
                    <div class="text-xs text-[var(--noro-muted)] flex items-center gap-2">
                      <span v-if="selectedActiveCape && currentSelectedCapeObj">
                        {{ grantedCapeIds.has(selectedActiveCape) ? 'Access Granted' : 'Access Not Granted' }}
                      </span>
                      <span v-else>No active cape assigned</span>
                    </div>
                  </div>
                </div>
                <UIcon name="i-lucide-chevron-down" class="size-5 text-[var(--noro-muted)] transition-transform duration-200" :class="capeDropdownOpen ? 'rotate-180' : ''" />
              </button>

              <!-- Dropdown Menu Popover -->
              <div
                v-if="capeDropdownOpen"
                class="absolute left-0 right-0 z-50 mt-2 max-h-64 overflow-y-auto rounded-lg border border-[var(--noro-border)] bg-[var(--noro-panel)] p-2 shadow-2xl space-y-1"
              >
                <!-- Option 0: Disabled -->
                <div
                  class="flex items-center justify-between rounded-lg p-2.5 cursor-pointer transition"
                  :class="!selectedActiveCape ? 'bg-[var(--noro-input)] border border-[var(--noro-cream)]' : 'hover:bg-[var(--noro-input)]'"
                  @click="selectedActiveCape = ''; capeDropdownOpen = false"
                >
                  <div class="flex items-center gap-3">
                    <div class="grid size-8 place-items-center rounded bg-[var(--noro-bg-deep)]">
                      <UIcon name="i-lucide-x" class="size-4 text-[var(--noro-muted)]" />
                    </div>
                    <span class="text-xs font-bold text-[var(--noro-text)]">No cape (Disabled)</span>
                  </div>
                  <UBadge v-if="!selectedActiveCape" color="primary" variant="subtle">Selected</UBadge>
                </div>

                <!-- Catalog Cape Options -->
                <div
                  v-for="cape in capes"
                  :key="cape.id"
                  class="flex items-center justify-between rounded-lg p-2.5 cursor-pointer transition"
                  :class="[
                    selectedActiveCape === cape.id ? 'bg-[var(--noro-input)] border border-[var(--noro-cream)]' : 'hover:bg-[var(--noro-input)]',
                    !grantedCapeIds.has(cape.id) ? 'opacity-60' : ''
                  ]"
                  @click="selectedActiveCape = cape.id; capeDropdownOpen = false"
                >
                  <div class="flex items-center gap-3">
                    <CapePreview :url="cape.url" class="h-8 shadow rounded" />
                    <div>
                      <div class="text-xs font-bold text-[var(--noro-text)]">{{ cape.name }}</div>
                      <div class="text-[10px] text-[var(--noro-muted)]">{{ Math.ceil(cape.size / 1024) }} KB</div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <UBadge :color="grantedCapeIds.has(cape.id) ? 'success' : 'warning'" variant="subtle" class="text-[10px]">
                      {{ grantedCapeIds.has(cape.id) ? 'Granted' : 'Locked' }}
                    </UBadge>
                    <UIcon v-if="selectedActiveCape === cape.id" name="i-lucide-check" class="size-4 text-[var(--noro-cream)]" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Granted Capes Grid Access -->
          <div class="noro-panel p-5 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-lg font-bold text-[var(--noro-text)]">Granted Capes Access</h2>
                <p class="text-xs text-[var(--noro-muted)]">Toggle capes from server catalog allowed for this player to choose in Cabinet & Launcher</p>
              </div>
              <span class="text-xs font-bold text-[var(--noro-cream)]">{{ grantedCapeIds.size }} / {{ capes.length }} granted</span>
            </div>

            <div v-if="capes.length" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
              <div
                v-for="cape in capes"
                :key="cape.id"
                class="group relative flex flex-col items-center justify-between rounded-lg border p-3 cursor-pointer transition"
                :class="grantedCapeIds.has(cape.id) ? 'border-[var(--noro-cream)] bg-[var(--noro-input)] shadow' : 'border-[var(--noro-border)] bg-[var(--noro-bg-deep)] opacity-60 hover:opacity-100'"
                @click="toggleCapeGrant(cape.id)"
              >
                <div class="w-full flex justify-between items-center mb-2">
                  <span class="truncate text-xs font-bold text-[var(--noro-text)]">{{ cape.name }}</span>
                  <UCheckbox :model-value="grantedCapeIds.has(cape.id)" @update:model-value="toggleCapeGrant(cape.id)" />
                </div>
                <CapePreview :url="cape.url" class="w-12 shadow" />
                <span class="mt-2 text-[10px] text-[var(--noro-muted)]">{{ grantedCapeIds.has(cape.id) ? 'Granted' : 'Locked' }}</span>
              </div>
            </div>
            <EmptyState v-else icon="i-lucide-flag" title="No capes in catalog" />
          </div>
        </section>
      </div>

      <!-- Tab 3: Moderation -->
      <div v-if="activeTab === 'moderation'" class="noro-panel p-5 space-y-4 max-w-xl">
        <h2 class="text-xl font-black text-[var(--noro-text)]">User Moderation</h2>
        <p class="text-sm text-[var(--noro-muted)]">Restrict player launcher login and server access</p>
        <input v-model="banReason" class="noro-input" placeholder="Enter ban reason...">
        <div class="flex flex-wrap gap-3">
          <AtomButton variant="danger" :loading="busy === 'ban'" icon="i-lucide-ban" @click="setBan(true)">Ban Player</AtomButton>
          <AtomButton variant="secondary" :loading="busy === 'ban'" icon="i-lucide-check" @click="setBan(false)">Unban Player</AtomButton>
        </div>
      </div>
    </div>

    <ImpersonateDialog
      v-if="user"
      v-model="showImpersonate"
      :user-id="id"
      :username="user.username"
    />
    <RequestLogsDialog
      v-if="user"
      v-model="showRequestLogs"
      :user-id="id"
      :username="user.username"
    />
  </NoroShell>
</template>
