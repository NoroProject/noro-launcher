<script setup lang="ts">
/**
 * Справочник игроков.
 *
 * Поиск и оба фильтра — серверные. Раньше страница тянула `?limit=500` и
 * фильтровала массив у себя: на пятьсот первом игроке список молча обрывался, а
 * «ничего не найдено» означало лишь «нет среди загруженных».
 */
import type { Role, UserProfile } from '~/types/api'

const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const statusFilter = ref<'all' | 'active' | 'banned'>('all')
const roleFilter = ref('all')

const list = usePagedList<UserProfile>('admin-users', '/api/admin/users', {
    params: () => ({
        banned: statusFilter.value === 'all' ? undefined : String(statusFilter.value === 'banned'),
        role: roleFilter.value === 'all' ? undefined : roleFilter.value,
    }),
})

// Смена фильтра — это новый запрос и первая страница: третья страница прежней
// выдачи к новому отбору отношения не имеет.
watch([statusFilter, roleFilter], () => list.goTo(1))

/** Роли берутся из своего справочника: по текущей странице их не собрать. */
const { data: roles } = await useAsyncData('admin-users-roles', () =>
    // Пустой список ролей в фильтре выглядит как «ролей не заведено».
    auth.requestList<Role>('/api/admin/roles'),
)
const roleNames = computed(() => (roles.value ?? []).map((r) => r.display_name || r.name))
</script>

<template>
    <NoroShell :title="t('nav-admin-users')" :subtitle="t('admin-users-subtitle')">
        <template #actions>
            <AtomButton
                icon="i-lucide-refresh-cw"
                variant="dark"
                :loading="list.pending.value"
                @click="list.refresh()"
            >
                {{ t('cabinet-apps-refresh') }}
            </AtomButton>
        </template>

        <UAlert
            v-if="list.error.value"
            class="mb-5"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="humanError(list.error.value)"
        />

        <div class="noro-panel mb-4 grid gap-3 p-4 md:grid-cols-[1fr_200px_200px]">
            <div>
                <label class="noro-label mb-2 block">{{ t('admin-users-search-label') }}</label>
                <input
                    v-model="list.search.value"
                    class="noro-input w-full"
                    :placeholder="t('admin-users-search-placeholder')"
                />
            </div>

            <div>
                <label class="noro-label mb-2 block">{{ t('admin-users-status-label') }}</label>
                <NoroSelect v-model="statusFilter" class="w-full">
                    <option value="all">{{ t('admin-users-status-all') }}</option>
                    <option value="active">{{ t('admin-users-status-active-only') }}</option>
                    <option value="banned">{{ t('admin-users-status-banned-only') }}</option>
                </NoroSelect>
            </div>

            <div>
                <label class="noro-label mb-2 block">{{ t('admin-users-role-label') }}</label>
                <NoroSelect v-model="roleFilter" class="w-full">
                    <option value="all">{{ t('admin-users-role-all') }}</option>
                    <option v-for="name in roleNames" :key="name" :value="name">{{ name }}</option>
                </NoroSelect>
            </div>
        </div>

        <section class="noro-panel overflow-hidden">
            <table v-if="list.items.value.length" class="noro-table">
                <thead>
                    <tr>
                        <th>{{ t('admin-users-player') }}</th>
                        <th>{{ t('admin-users-identity') }}</th>
                        <th>{{ t('admin-users-roles') }}</th>
                        <th>{{ t('admin-users-status') }}</th>
                        <th />
                    </tr>
                </thead>
                <tbody>
                    <AdminUserRow
                        v-for="user in list.items.value"
                        :key="user.id"
                        :user="user"
                    />
                </tbody>
            </table>

            <EmptyState
                v-else
                icon="i-lucide-users"
                :title="list.search.value ? t('paging-empty') : t('admin-users-empty-title')"
            />

            <NoroPager
                :page="list.page.value"
                :pages="list.pages.value"
                :total="list.total.value"
                :per-page="list.perPage"
                @go="list.goTo"
            />
        </section>
    </NoroShell>
</template>
