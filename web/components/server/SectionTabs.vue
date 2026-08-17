<script setup lang="ts">
import type { ServerSettingsTab } from "~/types/server-settings";

const model = defineModel<ServerSettingsTab>({ required: true });
const { t } = useT();

const tabs = computed<Array<{
    key: ServerSettingsTab;
    label: string;
    icon: string;
    text: string;
}>>(() => [
    {
        key: "server",
        label: t('admin-tab-profile'),
        icon: "i-lucide-user",
        text: t('admin-tab-profile-hint'),
    },
    {
        key: "mods",
        label: t('admin-tab-mods'),
        icon: "i-lucide-puzzle",
        text: t('admin-tab-mods-hint'),
    },
    {
        key: "build",
        label: t('admin-tab-build'),
        icon: "i-lucide-box",
        text: t('admin-tab-build-hint'),
    },
    {
        key: "instances",
        label: t('admin-tab-instances'),
        icon: "i-lucide-server",
        text: t('admin-tab-instances-hint'),
    },
    {
        key: "client",
        label: t('admin-tab-client'),
        icon: "i-lucide-monitor",
        text: t('admin-tab-client-hint'),
    },
]);
</script>

<template>
    <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
        <button
            v-for="tab in tabs"
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
