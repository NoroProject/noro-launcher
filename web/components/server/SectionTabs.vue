<script setup lang="ts">
import type { ServerSettingsTab } from "~/types/server-settings";

const model = defineModel<ServerSettingsTab>({ required: true });
const { t } = useT();

const auth = useAuth();

const tabs = computed<Array<{
    key: ServerSettingsTab;
    label: string;
    icon: string;
    text: string;
    perms: string[];
}>>(() => [
    {
        key: "server",
        label: t('admin-tab-profile'),
        icon: "i-lucide-user",
        text: t('admin-tab-profile-hint'),
        perms: ['noro.admin.servers.view'],
    },
    {
        key: "mods",
        label: t('admin-tab-mods'),
        icon: "i-lucide-puzzle",
        text: t('admin-tab-mods-hint'),
        perms: ['noro.admin.mods.view'],
    },
    {
        key: "build",
        label: t('admin-tab-build'),
        icon: "i-lucide-box",
        text: t('admin-tab-build-hint'),
        perms: ['noro.admin.builds.view'],
    },
    {
        key: "instances",
        label: t('admin-tab-instances'),
        icon: "i-lucide-server",
        text: t('admin-tab-instances-hint'),
        perms: ['noro.admin.servers.agents', 'noro.admin.wrapper.view'],
    },
    {
        key: "client",
        label: t('admin-tab-client'),
        icon: "i-lucide-monitor",
        text: t('admin-tab-client-hint'),
        perms: ['noro.admin.servers.view', 'noro.admin.builds.view'],
    },
]);

const visibleTabs = computed(() => tabs.value.filter(t => !t.perms.length || auth.hasAny(...t.perms)));
</script>

<template>
    <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
        <button
            v-for="tab in visibleTabs"
            :key="tab.key"
            type="button"
            class="noro-tab"
            :class="{ 'noro-tab-active': model === tab.key }"
            @click="model = tab.key"
        >
            <UIcon :name="tab.icon" class="size-5 shrink-0" />
            <span class="min-w-0 text-left">
                <span class="block font-bold">{{ tab.label }}</span>
                <span class="block truncate text-xs opacity-70">
                    {{ tab.text }}
                </span>
            </span>
        </button>
    </div>
</template>
