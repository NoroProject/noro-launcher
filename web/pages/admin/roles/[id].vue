<script setup lang="ts">
import type { Role, ServerRow } from '~/types/api'
import type { PermissionEntry } from '~/types/permissions'
import { toPermissionEntries } from '~/types/permissions'

const route = useRoute()
const auth = useAuth()
await auth.loadMe()

const id = computed(() => String(route.params.id))
const { data: roles, refresh } = await useAsyncData('admin-role-edit-list', () =>
  auth.request<Role[]>('/api/admin/roles'), { default: () => [] }
)
const { data: servers } = await useAsyncData('admin-role-servers', () =>
  auth.request<ServerRow[]>('/api/admin/servers'), { default: () => [] }
)
const role = computed(() => roles.value.find(item => item.id === id.value))
const permissions = computed(() =>
  toPermissionEntries(role.value?.permission_grants, role.value?.permissions || [])
)
const inherited = computed(() => role.value?.inherited_permissions || [])
/** Родителем может стать любая роль, кроме этой: цикл мастер всё равно отклонит. */
const parentOptions = computed(() => roles.value.filter(item => item.id !== id.value))
const parentName = computed(() =>
  roles.value.find(item => item.id === role.value?.parent_id)?.display_name || ''
)
const grants = usePermissionGrants(computed(() => `/api/admin/roles/${id.value}`))
const form = reactive({
  display_name: '',
  color: '#e85aa5',
  is_default: false,
  sort_order: 0,
  lp_group: '',
  icon: '',
  parent_id: ''
})
const busy = ref<string | null>(null)

watchEffect(() => {
  if (!role.value) return
  Object.assign(form, {
    display_name: role.value.display_name,
    color: role.value.color || '#e85aa5',
    is_default: role.value.is_default,
    sort_order: role.value.sort_order || 0,
    lp_group: role.value.lp_group || '',
    icon: role.value.icon || '',
    parent_id: role.value.parent_id || ''
  })
})

async function run(name: string, action: () => Promise<void>) {
  busy.value = name
  try {
    await action()
    await refresh()
  } finally {
    busy.value = null
  }
}

async function save() {
  // Пустая строка из <select> — это «без родителя», а не роль с пустым id.
  const body = { ...form, parent_id: form.parent_id || null }
  await run('save', () => auth.request(`/api/admin/roles/${id.value}`, { method: 'PUT', body }))
}

async function removeRole() {
  await run('delete', async () => {
    await auth.request(`/api/admin/roles/${id.value}`, { method: 'DELETE' })
    await navigateTo('/admin/roles')
  })
}

async function addPermission(entries: PermissionEntry[]) {
  await run(entries.length === 1 ? `perm-${entries[0]!.permission}` : 'perm', () =>
    grants.addMany(entries)
  )
}

async function removePermission(entries: PermissionEntry[]) {
  await run(`perm-${entries[0]?.permission}`, () => grants.removeMany(entries))
}
</script>

<template>
  <NoroShell :title="role?.display_name || 'ROLE'" :subtitle="role?.name">
    <template #actions>
      <AtomButton variant="ghost" icon="i-lucide-arrow-left" :to="'/admin/roles'">Back</AtomButton>
    </template>

    <EmptyState v-if="!role" icon="i-lucide-search-x" title="Role not found" />

    <div v-else class="grid gap-5 xl:grid-cols-[380px_1fr]">
      <form class="noro-panel grid gap-3 p-5" @submit.prevent="save">
        <label><span class="noro-label">Display name</span><input v-model="form.display_name" class="noro-input" required></label>
        <label><span class="noro-label">Color</span><input v-model="form.color" class="noro-input" type="color"></label>
        <label><span class="noro-label">Order</span><input v-model.number="form.sort_order" class="noro-input" type="number"></label>
        <label>
          <span class="noro-label">Icon</span>
          <input v-model="form.icon" class="noro-input" maxlength="8" placeholder="★">
          <span class="mt-2 block text-xs text-[var(--noro-muted)]">
            A single character shown next to the name. Unicode works everywhere &mdash;
            in the cabinet, in the launcher and in game chat.
          </span>
        </label>
        <label>
          <span class="noro-label">Inherits from</span>
          <select v-model="form.parent_id" class="noro-input noro-select">
            <option value="">Nothing — own permissions only</option>
            <option v-for="item in parentOptions" :key="item.id" :value="item.id">
              {{ item.display_name }}
            </option>
          </select>
          <span class="mt-2 block text-xs text-[var(--noro-muted)]">
            Everything the parent grants applies here too, all the way up the chain.
            A role cannot inherit from one that already inherits from it.
          </span>
        </label>
        <label>
          <span class="noro-label">LuckPerms group</span>
          <input v-model="form.lp_group" class="noro-input" placeholder="vip">
          <span class="mt-2 block text-xs text-[var(--noro-muted)]">
            Links this role to a group in game. Leave empty if the role should not
            reach the server. Two roles cannot point at the same group.
          </span>
        </label>
        <UCheckbox v-model="form.is_default" label="Default role" />
        <div class="flex gap-2">
          <AtomButton
            variant="primary"
            icon="i-lucide-save"
            :loading="busy === 'save'"
            type="submit"
            :disabled="busy === 'save'"
          >
            Save
          </AtomButton>
          <AtomButton variant="danger" :loading="busy === 'delete'" icon="i-lucide-trash-2" @click="removeRole">Delete</AtomButton>
        </div>
      </form>

      <div class="grid gap-5">
        <AdminPermissionEditor
          title="Role permissions"
          subtitle="Everyone in this role gets them. Pick the builds a permission applies to, or all of them."
          :entries="permissions"
          :servers="servers"
          :busy="busy"
          @add="addPermission"
          @remove="removePermission"
        />
        <AdminInheritedPermissions :permissions="inherited" :parent-name="parentName" />
      </div>
    </div>
  </NoroShell>
</template>
