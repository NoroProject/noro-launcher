<script setup lang="ts">
import type { Role } from "~/types/api";

defineProps<{
    roles: Role[];
    togglingRole: string | null;
    hasAccess: (role: Role) => boolean;
}>();

defineEmits<{
    toggle: [role: Role];
}>();

function isAdminRole(role: Role) {
    return role.permissions.includes("*") || role.permissions.includes("noro.admin.*");
}
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div class="border-b border-[var(--noro-border)] p-5">
            <h2 class="text-base font-black text-white">Role visibility</h2>
            <p class="text-sm text-[var(--noro-muted)]">
                Select roles that can see and join this limited server.
            </p>
        </div>
        <div class="divide-y divide-[var(--noro-border)]">
            <div
                v-for="role in roles"
                :key="role.id"
                class="flex items-center justify-between gap-4 px-5 py-4 transition hover:bg-white/5"
            >
                <div class="flex min-w-0 items-center gap-3">
                    <span
                        class="size-3 rounded-full bg-[var(--noro-blue)]"
                        :style="{ backgroundColor: role.color || 'var(--noro-blue)' }"
                    />
                    <div class="min-w-0">
                        <div class="truncate text-sm font-bold text-white">
                            {{ role.display_name }}
                        </div>
                        <div class="truncate text-xs text-[var(--noro-muted)]">
                            {{ isAdminRole(role) ? "Admin bypass" : role.name }}
                        </div>
                    </div>
                </div>
                <UTooltip :text="isAdminRole(role) ? 'Admin roles bypass limits' : ''">
                    <UCheckbox
                        :model-value="hasAccess(role)"
                        :disabled="togglingRole === role.id || (hasAccess(role) && isAdminRole(role))"
                        @update:model-value="$emit('toggle', role)"
                    />
                </UTooltip>
            </div>
        </div>
    </section>
</template>
