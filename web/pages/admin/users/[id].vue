<script setup lang="ts">
import UserGameActionsPanel from '~/components/admin/UserGameActionsPanel.vue'
import type { Role, ServerRow, UserProfile } from '~/types/api'
import type { CapeRow } from '~/types/cape'
import type { PermissionEntry } from '~/types/permissions'
import { toPermissionEntries } from '~/types/permissions'

const route = useRoute()
const auth = useAuth()
const { t } = useT()
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
type Tab = 'profile' | 'skin_capes' | 'support' | 'moderation' | 'game_actions'

const tabs = computed<{ id: Tab, label: string, icon: string, perms: string[] }[]>(() => [
  { id: 'profile', label: t('admin-users-tab-profile'), icon: 'i-lucide-user', perms: ['noro.admin.users.view'] },
  { id: 'skin_capes', label: t('admin-users-tab-skins'), icon: 'i-lucide-sparkles', perms: ['noro.admin.users.skin', 'noro.admin.users.capes'] },
  { id: 'support', label: t('admin-users-tab-support'), icon: 'i-lucide-life-buoy', perms: ['noro.admin.support.logs', 'noro.admin.users.launcher'] },
  { id: 'moderation', label: t('admin-users-tab-mod'), icon: 'i-lucide-gavel', perms: ['noro.mod.punish.view', 'noro.admin.users.notes.view'] },
  { id: 'game_actions', label: t('admin-users-tab-game-actions') !== 'admin-users-tab-game-actions' ? t('admin-users-tab-game-actions') : 'Игровые действия', icon: 'i-lucide-gamepad-2', perms: ['noro.admin.users.view'] },
])
const visibleTabs = computed(() => tabs.value.filter(tab => auth.hasAny(...tab.perms)))
const activeTab = ref<Tab>('profile')

watchEffect(() => {
  if (visibleTabs.value.length && !visibleTabs.value.some(tab => tab.id === activeTab.value)) {
    activeTab.value = visibleTabs.value[0]!.id
  }
})

const can = (perm: string) => auth.hasPermission(perm)

const permissions = computed(() =>
  toPermissionEntries(user.value?.permission_grants, user.value?.permissions || [])
)
const grants = usePermissionGrants(computed(() => `/api/admin/users/${id.value}`))

const selectedActiveCape = ref('')
const capeDropdownOpen = ref(false)
const currentSelectedCapeObj = computed(() => capes.value.find(c => c.id === selectedActiveCape.value))
const grantedCapeIds = ref<Set<string>>(new Set())
const busy = ref<string | null>(null)

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
      <AtomButton variant="dark" icon="i-lucide-arrow-left" to="/admin/users">{{ t('nav-admin-users') }}</AtomButton>
    </template>

    <EmptyState v-if="!user" icon="i-lucide-search-x" :title="t('admin-users-not-found')" />

    <div v-else class="space-y-6">
      <div class="noro-panel flex flex-wrap gap-2 p-2">
        <button
          v-for="tab in visibleTabs"
          :key="tab.id"
          class="flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-bold transition"
          :class="activeTab === tab.id ? 'bg-[var(--noro-input)] text-[var(--noro-cream)] shadow' : 'text-[var(--noro-muted)] hover:text-[var(--noro-text)]'"
          @click="activeTab = tab.id"
        >
          <UIcon :name="tab.icon" class="size-4" />
          <span>{{ tab.label }}</span>
        </button>
      </div>

      <div v-if="activeTab === 'profile'" class="grid gap-5 xl:grid-cols-[360px_1fr]">
        <div class="space-y-5">
        <div class="noro-panel h-fit p-5 space-y-4">
          <div class="flex items-center gap-4">
            <img v-if="user.discord_avatar" :src="user.discord_avatar" alt="" class="size-16 rounded-lg object-cover">
            <div v-else class="grid size-16 place-items-center rounded-lg bg-[var(--noro-magenta)] text-2xl font-black text-[var(--noro-white)]">{{ user.username.slice(0, 1).toUpperCase() }}</div>
            <div class="min-w-0">
              <h2 class="truncate text-xl font-black text-[var(--noro-text)]">{{ user.username }}</h2>
              <p class="truncate text-sm text-[var(--noro-muted)]">{{ user.discord_username }}</p>
              <UBadge class="mt-2" :color="user.banned ? 'error' : 'success'" variant="subtle">{{ user.banned ? t('admin-users-banned') : t('admin-users-active') }}</UBadge>
            </div>
          </div>
          <div class="rounded-lg bg-[var(--noro-input)] p-3">
            <span class="text-xs uppercase tracking-wider text-[var(--noro-muted)] font-bold block mb-1">UUID</span>
            <code class="break-all text-xs text-[var(--noro-text)]">{{ user.uuid }}</code>
          </div>
        </div>
        <UserLauncherPanel
          v-if="can('noro.admin.users.launcher')"
          v-model:online="launcherOnline"
          :user-id="id"
          @impersonate="showImpersonate = true"
          @request-logs="showRequestLogs = true"
        />
        </div>

        <div class="grid gap-5">
          <AdminUserRoles
            v-if="can('noro.admin.users.roles')"
            :roles="roles"
            :user-roles="user.roles"
            :busy="busy"
            @add="addRole"
            @remove="removeRole"
          />
          <AdminPermissionEditor
            :title="t('admin-users-direct-perms')"
            :subtitle="t('admin-users-direct-perms-hint')"
            :entries="permissions"
            :servers="servers"
            :busy="busy"
            @add="addPermission"
            @remove="removePermission"
          />
        </div>
      </div>

      <div v-if="activeTab === 'skin_capes'" class="grid gap-5 xl:grid-cols-[380px_1fr]">
        <section v-if="can('noro.admin.users.skin')" class="space-y-4">
          <div class="noro-panel p-5 space-y-4">
            <h3 class="noro-label">{{ t('admin-users-skin-preview') }}</h3>
            <SkinPreview3D :skin-url="user.skin_url" :cape-url="user.cape_url" />
            
            <div class="rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] p-4 space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-[var(--noro-muted)] uppercase">{{ t('admin-users-custom-skin') }}</span>
                <UBadge :color="user.skin_url ? 'primary' : 'neutral'" variant="subtle">{{ user.skin_url ? t('admin-users-uploaded') : t('admin-users-default-skin') }}</UBadge>
              </div>

              <div class="flex items-center gap-2">
                <input ref="skinInput" type="file" accept="image/png" class="hidden" @change="onSkinFilePicked">
                <AtomButton
                  variant="primary"
                  icon="i-lucide-upload"
                  :disabled="busy === 'upload-skin'"
                  @click="triggerSkinPicker"
                >
                  {{ t('admin-users-upload-skin') }}
                </AtomButton>

                <AtomButton
                  v-if="user.skin_url"
                  variant="danger"
                  icon="i-lucide-trash-2"
                  :disabled="busy === 'remove-skin'"
                  @click="removeSkinForUser"
                >
                  {{ t('admin-users-reset-skin') }}
                </AtomButton>
              </div>
            </div>

          </div>
        </section>

        <section class="space-y-5">
        <div v-if="can('noro.admin.users.skin')" class="noro-panel p-5 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <h3 class="noro-label">{{ t('admin-users-presets-title') }}</h3>
              <p class="text-xs text-[var(--noro-muted)]">{{ t('admin-users-presets-subtitle') }}</p>
            </div>
            <UBadge color="primary" variant="subtle">{{ t('admin-capes-presets-count', { count: skinPresets.length }) }}</UBadge>
          </div>

          <div v-if="skinPresets.length" class="grid gap-3 sm:grid-cols-3 lg:grid-cols-4">
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

              <div class="mb-2 flex items-center justify-center overflow-hidden rounded border border-[var(--noro-border)] bg-[var(--noro-input)] py-2">
                <SkinCard3D :skin-url="preset.skin_url" mode="body" :scale="6" />
              </div>

              <div>
                <UBadge v-if="user.skin_url === preset.skin_url" color="primary" variant="subtle" class="w-full justify-center text-[10px]">
                  {{ t('admin-users-active') }}
                </UBadge>
                <AtomButton
                  v-else
                  variant="secondary"
                  size="sm"
                  class="w-full justify-center text-[10px]"
                  :disabled="busy === 'select-preset'"
                  @click="selectPresetForUser(preset.skin_url)"
                >
                  {{ t('admin-capes-equip') }}
                </AtomButton>
              </div>
            </div>
          </div>
          <EmptyState v-else icon="i-lucide-images" title="No skin presets" :text="t('admin-capes-empty-presets')" />
        </div>

          <div v-if="can('noro.admin.users.capes')" class="noro-panel p-5 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-lg font-bold text-[var(--noro-text)]">{{ t('admin-users-active-cape') }}</h2>
                <p class="text-xs text-[var(--noro-muted)]">{{ t('admin-users-active-cape-subtitle') }}</p>
              </div>
              <AtomButton
                variant="primary"
                icon="i-lucide-save"
                :disabled="busy === 'save-capes'"
                @click="saveCapesAccess"
              >
                {{ t('cabinet-save') }}
              </AtomButton>
            </div>

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
                      {{ selectedActiveCape && currentSelectedCapeObj ? currentSelectedCapeObj.name : t('admin-users-no-cape') }}
                    </div>
                    <div class="text-xs text-[var(--noro-muted)] flex items-center gap-2">
                      <span v-if="selectedActiveCape && currentSelectedCapeObj">
                        {{ grantedCapeIds.has(selectedActiveCape) ? t('admin-capes-access-granted') : t('admin-capes-access-not-granted') }}
                      </span>
                    </div>
                  </div>
                </div>
                <UIcon name="i-lucide-chevron-down" class="size-5 text-[var(--noro-muted)] transition-transform duration-200" :class="capeDropdownOpen ? 'rotate-180' : ''" />
              </button>

              <div
                v-if="capeDropdownOpen"
                class="absolute left-0 right-0 z-50 mt-2 max-h-64 overflow-y-auto rounded-lg border border-[var(--noro-border)] bg-[var(--noro-panel)] p-2 shadow-2xl space-y-1"
              >
                <div
                  class="flex items-center justify-between rounded-lg p-2.5 cursor-pointer transition"
                  :class="!selectedActiveCape ? 'bg-[var(--noro-input)] border border-[var(--noro-cream)]' : 'hover:bg-[var(--noro-input)]'"
                  @click="selectedActiveCape = ''; capeDropdownOpen = false"
                >
                  <div class="flex items-center gap-3">
                    <div class="grid size-8 place-items-center rounded bg-[var(--noro-bg-deep)]">
                      <UIcon name="i-lucide-x" class="size-4 text-[var(--noro-muted)]" />
                    </div>
                    <span class="text-xs font-bold text-[var(--noro-text)]">{{ t('admin-users-no-cape') }}</span>
                  </div>
                  <UBadge v-if="!selectedActiveCape" color="primary" variant="subtle">{{ t('admin-capes-selected') }}</UBadge>
                </div>

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
                      {{ grantedCapeIds.has(cape.id) ? t('admin-capes-granted') : t('admin-capes-locked') }}
                    </UBadge>
                    <UIcon v-if="selectedActiveCape === cape.id" name="i-lucide-check" class="size-4 text-[var(--noro-cream)]" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-if="can('noro.admin.users.capes')" class="noro-panel p-5 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-lg font-bold text-[var(--noro-text)]">{{ t('admin-users-granted-capes') }}</h2>
                <p class="text-xs text-[var(--noro-muted)]">{{ t('admin-users-granted-capes-subtitle') }}</p>
              </div>
              <span class="text-xs font-bold text-[var(--noro-cream)]">{{ t('admin-capes-count-granted', { count: grantedCapeIds.size, total: capes.length }) }}</span>
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
                <span class="mt-2 text-[10px] text-[var(--noro-muted)]">{{ grantedCapeIds.has(cape.id) ? t('admin-capes-granted') : t('admin-capes-locked') }}</span>
              </div>
            </div>
            <EmptyState v-else icon="i-lucide-flag" title="No capes in catalog" />
          </div>
        </section>
      </div>

      <div v-if="activeTab === 'support'" class="grid gap-5 xl:grid-cols-2">
        <SupportBundlesPanel v-if="can('noro.admin.support.logs')" :user-id="id" @request-logs="showRequestLogs = true" />
        <DiagnosticsCard :user-id="id" :online="launcherOnline" />
      </div>

      <div v-if="activeTab === 'game_actions'" class="space-y-5">
        <UserGameActionsPanel v-if="user" :username="user.username" :user-id="id" :target-uuid="user.uuid" />
      </div>

      <div v-if="activeTab === 'moderation'" class="space-y-5">
        <PunishmentsPanel v-if="can('noro.mod.punish.view')" :user-id="id" />
        <AdminFreezeUserBlock v-if="can('noro.mod.freeze')" :user-id="id" :frozen="user?.frozen" @updated="refreshUser" />
        <UserNotesPanel v-if="can('noro.admin.users.notes.view')" :user-id="id" />
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
