<script setup lang="ts">
import type { ServerSettingsTab } from "~/types/server-settings";

const model = defineModel<ServerSettingsTab>({ required: true });

const tabs: Array<{
    key: ServerSettingsTab;
    label: string;
    icon: string;
    text: string;
}> = [
    {
        key: "server",
        label: "Profile",
        icon: "i-lucide-server",
        text: "Name, order, and visibility in the launcher",
    },
    {
        key: "client",
        label: "Client",
        icon: "i-lucide-monitor",
        text: "Minecraft version and loader defaults",
    },
];
</script>

<template>
    <div class="grid gap-3 md:grid-cols-2">
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
