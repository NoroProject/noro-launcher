<script setup lang="ts">
/** Строка справочника игроков: аватар с головой скина, ник, роли, статус. */
import type { UserProfile } from '~/types/api'

defineProps<{ user: UserProfile }>()
const { t } = useT()

const masterUrl = useRuntimeConfig().public.masterUrl

function headUrl(skinUrl?: string | null) {
    const params = new URLSearchParams({ mode: 'flat-head', scale: '8' })
    if (skinUrl) params.set('url', skinUrl)
    return `${masterUrl}/api/textures/renders?${params.toString()}`
}
</script>

<template>
    <tr>
        <td>
            <div class="flex items-center gap-3">
                <div class="relative size-10 flex-shrink-0">
                    <img
                        :src="identityAvatar(user) || '/default-avatar.png'"
                        class="size-10 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] object-cover"
                        alt="Avatar"
                    />
                    <div
                        class="absolute -bottom-1 -right-1 size-5 overflow-hidden rounded border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] shadow"
                    >
                        <img
                            :src="headUrl(user.skin_url)"
                            class="size-full object-contain"
                            alt="Skin Head"
                        />
                    </div>
                </div>
                <div>
                    <div class="font-semibold text-[var(--noro-text)]">{{ user.username }}</div>
                    <code class="text-xs text-[var(--noro-muted)]">{{ user.uuid }}</code>
                </div>
            </div>
        </td>
        <td>{{ identityHandle(user) || "—" }}</td>
        <td>
            <div class="flex flex-wrap gap-1">
                <UBadge v-for="role in user.roles" :key="role.id" color="neutral" variant="subtle">
                    {{ role.display_name }}
                </UBadge>
            </div>
        </td>
        <td>
            <UBadge :color="user.banned ? 'error' : 'success'" variant="subtle">
                {{ user.banned ? t('admin-users-banned') : t('admin-users-active') }}
            </UBadge>
        </td>
        <td class="text-right">
            <AtomButton
                variant="dark"
                icon="i-lucide-settings"
                :to="`/admin/users/${user.id}`"
                class="!min-h-8 !min-w-8 !px-2"
            />
        </td>
    </tr>
</template>
