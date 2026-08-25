<script setup lang="ts">
import type { Role, ServerRow } from '~/types/api'
import type { PermissionEntry } from '~/types/permissions'
import { toPermissionEntries } from '~/types/permissions'

const route = useRoute()
const auth = useAuth()
const notify = useNotify()
const { t } = useT()

/** Есть ли у роли своя картинка плашки: от этого зависит кнопка «Убрать». */
const hasBadgeImage = computed(() => Boolean((role.value as { badge_sha1?: string } | null)?.badge_sha1))
const can = (perm: string) => auth.hasPermission(perm)
await auth.loadMe()

const id = computed(() => String(route.params.id))
const { data: roles, refresh } = await useAsyncData('admin-role-edit-list', () =>
  auth.requestList<Role>('/api/admin/roles'), { default: () => [] }
)
const { data: servers } = await useAsyncData('admin-role-servers', () =>
  auth.requestList<ServerRow>('/api/admin/servers'), { default: () => [] }
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
  prefix: '',
  suffix: '',
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
    prefix: role.value.prefix || '',
    suffix: role.value.suffix || '',
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
  const body = { ...form, parent_id: form.parent_id || null }
  // Раньше кнопка просто гасла на полсекунды, и понять, сохранилось ли, было
  // нельзя: страница выглядит после сохранения ровно так же, как до.
  try {
    await run('save', () => auth.request(`/api/admin/roles/${id.value}`, { method: 'PUT', body }))
    notify.ok(t('admin-role-saved'), t('admin-role-saved-hint'))
  } catch (e) {
    notify.fail(e)
  }
}

async function removeRole() {
  await run('delete', async () => {
    await auth.request(`/api/admin/roles/${id.value}`, { method: 'DELETE' })
    await navigateTo(adminLink.roles())
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
  <NoroShell :title="role?.display_name || t('admin-roles-title')" :subtitle="role?.name">
    <template #actions>
      <AtomButton variant="ghost" icon="i-lucide-arrow-left" :to="'/admin/roles'">{{ t('admin-role-back') }}</AtomButton>
    </template>

    <EmptyState v-if="!role" icon="i-lucide-search-x" :title="t('cabinet-roles-none-title')" />

    <!-- items-start: без него колонки тянутся до высоты самой длинной, и под
         коротким списком прав остаётся полэкрана пустоты. -->
    <div v-else class="grid items-start gap-5 xl:grid-cols-[minmax(0,420px)_1fr]">
      <!-- Три раздела вместо одной колонки из десяти полей: у них разный
           смысл, и раньше «цвет» стоял вплотную к «группе LuckPerms», хотя
           одно про внешность, другое про связь с игрой. -->
      <form class="grid gap-4" @submit.prevent="save">
        <section class="noro-panel grid gap-3 p-5">
          <h3 class="text-sm font-bold text-[var(--noro-text)]">{{ t('admin-role-section-look') }}</h3>
          <label><span class="noro-label">{{ t('admin-roles-display-name') }}</span><input v-model="form.display_name" class="noro-input" required></label>
          <label><span class="noro-label">{{ t('admin-roles-color') }}</span><input v-model="form.color" class="noro-input" type="color"></label>
          <label>
            <span class="noro-label">{{ t('admin-roles-order') }}</span>
            <input v-model.number="form.sort_order" class="noro-input" type="number">
          </label>
        </section>

        <section class="noro-panel grid gap-3 p-5">
          <h3 class="text-sm font-bold text-[var(--noro-text)]">{{ t('admin-role-section-game') }}</h3>
          <label>
            <span class="noro-label">{{ t('admin-role-prefix-label') }}</span>
            <input v-model="form.prefix" class="noro-input font-mono" maxlength="64" placeholder="&8[&cADMIN&8] ">
            <span class="mt-2 block text-xs text-[var(--noro-muted)]">{{ t('admin-role-prefix-hint') }}</span>
          </label>
          <label>
            <span class="noro-label">{{ t('admin-role-suffix-label') }}</span>
            <input v-model="form.suffix" class="noro-input font-mono" maxlength="64" placeholder=" &7★">
            <span class="mt-2 block text-xs text-[var(--noro-muted)]">{{ t('admin-role-suffix-hint') }}</span>
          </label>
          <label>
            <span class="noro-label">Icon</span>
            <input v-model="form.icon" class="noro-input" maxlength="8" placeholder="★">
            <span class="mt-2 block text-xs text-[var(--noro-muted)]">{{ t('admin-role-icon-hint') }}</span>
          </label>

          <AdminPrefixBadge :text="form.prefix" :fallback="form.display_name" :color="form.color" />
          <AdminRoleBadgeUpload :role-id="id" :has-image="hasBadgeImage" @changed="refresh()" />
        </section>

        <section class="noro-panel grid gap-3 p-5">
          <h3 class="text-sm font-bold text-[var(--noro-text)]">{{ t('admin-role-section-rights') }}</h3>
          <label>
            <span class="noro-label">{{ t('admin-role-inherits-label') }}</span>
            <NoroSelect v-model="form.parent_id">
              <option value="">{{ t('admin-role-inherits-none') }}</option>
              <option v-for="item in parentOptions" :key="item.id" :value="item.id">{{ item.display_name }}</option>
            </NoroSelect>
            <span class="mt-2 block text-xs text-[var(--noro-muted)]">{{ t('admin-role-inherits-hint') }}</span>
          </label>
          <label>
            <span class="noro-label">{{ t('admin-role-lp-label') }}</span>
            <input v-model="form.lp_group" class="noro-input" placeholder="vip">
            <span class="mt-2 block text-xs text-[var(--noro-muted)]">{{ t('admin-role-lp-hint') }}</span>
          </label>
          <UCheckbox v-model="form.is_default" :label="t('admin-roles-is-default')" />
        </section>

        <div class="flex gap-2">
          <AtomButton
            v-if="can('noro.admin.roles.edit')"
            variant="primary"
            icon="i-lucide-save"
            :loading="busy === 'save'"
            type="submit"
            :disabled="busy === 'save'"
          >{{ t('cabinet-save') }}</AtomButton>
          <AtomButton v-if="can('noro.admin.roles.edit')" variant="danger" :loading="busy === 'delete'" icon="i-lucide-trash-2" @click="removeRole">{{ t('admin-blocklist-act-delete') }}</AtomButton>
        </div>
      </form>

      <div class="grid content-start gap-5">
        <AdminPermissionEditor
          :title="t('admin-role-perms-title')"
          :subtitle="t('admin-role-perms-subtitle')"
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
