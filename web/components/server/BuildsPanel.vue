<script setup lang="ts">
import type { BuildRow } from "~/types/api";

defineProps<{
    builds: BuildRow[];
    serverId: string;
    pending: boolean;
    error: unknown;
}>();

defineEmits<{
    refresh: [];
    create: [];
}>();
</script>

<template>
    <section class="grid gap-4">
        <div class="flex flex-wrap items-end justify-between gap-4">
            <div>
                <h2 class="text-xl font-black text-white">Builds</h2>
                <p class="text-sm text-[var(--noro-muted)]">
                    Manage game versions, files, mods, and publish state.
                </p>
            </div>
        </div>

        <UAlert
            v-if="error"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="humanError(error)"
        />

        <div class="noro-panel overflow-hidden">
            <table v-if="builds.length" class="noro-table">
                <thead>
                    <tr>
                        <th>Build</th>
                        <th>Minecraft</th>
                        <th>Modloader</th>
                        <th>Status</th>
                        <th />
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="build in builds" :key="build.id">
                        <td class="font-black text-white">{{ build.version }}</td>
                        <td>{{ build.mc_version }}</td>
                        <td>{{ build.modloader }} {{ build.modloader_version }}</td>
                        <td>
                            <UBadge
                                :color="build.published ? 'success' : 'neutral'"
                                variant="subtle"
                            >
                                {{ build.published ? "published" : "draft" }}
                            </UBadge>
                        </td>
                        <td class="text-right">
                            <AtomButton
                                :to="`/admin/servers/${serverId}?tab=build`"
                                icon="i-lucide-settings"
                                variant="ghost"
                            />
                        </td>
                    </tr>
                </tbody>
            </table>
            <EmptyState
                v-else
                icon="i-lucide-package"
                title="No builds yet"
                text="Create the first build from the toolbar."
            />
        </div>
    </section>
</template>
