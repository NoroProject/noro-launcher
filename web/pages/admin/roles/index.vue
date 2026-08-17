<script setup lang="ts">
import type { Role } from '~/types/api'

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)

const notify = useNotify()
await auth.loadMe()

const { data: roles, refresh, pending } = await useAsyncData('admin-roles', () =>
  auth.request<Role[]>('/api/admin/roles'), { default: () => [] }
)

const form = reactive({ name: '', display_name: '', color: '#e85aa5', is_default: false, sort_order: 0 })
const creating = ref(false)
const showCreate = ref(false)

async function createRole() {
  creating.value = true
  try {
    await auth.request('/api/admin/roles', { method: 'POST', body: form })
    Object.assign(form, { name: '', display_name: '', color: '#e85aa5', is_default: false, sort_order: 0 })
    await refresh()
    showCreate.value = false
    notify.ok()
  } catch (e) {
    notify.fail(e)
  } finally {
    creating.value = false
  }
}
</script>

<template>
  <NoroShell :title="t('admin-roles-title')" :subtitle="t('admin-roles-subtitle')">
    <template #actions>
      <AtomButton
        icon="i-lucide-refresh-cw"
        variant="dark"
        :loading="pending"
        @click="refresh()"
      >
        {{ t('cabinet-apps-refresh') }}
      </AtomButton>
      <AtomButton v-if="can('noro.admin.roles.edit')" variant="primary" icon="i-lucide-plus" @click="showCreate = true">{{ t('admin-roles-new-role') }}</AtomButton>
    </template>

    <section class="noro-panel overflow-hidden">
      <table v-if="roles?.length" class="noro-table">
        <thead>
          <tr>
            <th>{{ t('admin-roles-col-role') }}</th>
            <th>{{ t('admin-roles-col-perms') }}</th>
            <th>{{ t('admin-roles-col-default') }}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="role in roles" :key="role.id">
            <td>
              <div class="flex items-center gap-2">
                <span class="size-2 rounded-full" :style="{ backgroundColor: role.color || 'var(--noro-magenta)' }" />
                <span class="font-semibold text-[var(--noro-text)]">{{ role.display_name }}</span>
              </div>
              <code class="text-xs text-[var(--noro-muted)]">{{ role.name }}</code>
            </td>
            <td>{{ role.permissions.length }}</td>
            <td>{{ role.is_default ? t('admin-roles-yes') : t('admin-roles-no') }}</td>
            <td class="text-right"><AtomButton variant="ghost" :to="`/admin/roles/${role.id}`" icon="i-lucide-settings" size="sm" /></td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-else icon="i-lucide-shield" :title="t('cabinet-roles-none-title')" :text="t('cabinet-roles-none-text')" />
    </section>

    <AtomModal v-model="showCreate" :title="t('admin-roles-modal-title')" :subtitle="t('admin-roles-modal-subtitle')">
      <form class="grid gap-3" @submit.prevent="createRole">
        <label><span class="noro-label">{{ t('admin-roles-name') }}</span><input v-model="form.name" class="noro-input" required></label>
        <label><span class="noro-label">{{ t('admin-roles-display-name') }}</span><input v-model="form.display_name" class="noro-input" required></label>
        <label><span class="noro-label">{{ t('admin-roles-color') }}</span><input v-model="form.color" class="noro-input" type="color"></label>
        <label><span class="noro-label">{{ t('admin-roles-order') }}</span><input v-model.number="form.sort_order" class="noro-input" type="number"></label>
        <UCheckbox v-model="form.is_default" :label="t('admin-roles-is-default')" />
        <div class="flex justify-end gap-3 pt-2">
          <AtomButton variant="secondary" @click="showCreate = false">{{ t('web-rules-cancel') }}</AtomButton>
          <AtomButton variant="primary" icon="i-lucide-plus" :disabled="creating" type="submit">{{ t('admin-notes-add') }}</AtomButton>
        </div>
      </form>
    </AtomModal>
  </NoroShell>
</template>
