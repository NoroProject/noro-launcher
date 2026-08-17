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

const { t } = useT();
</script>

<template>
    <section class="grid gap-4">
        <div class="flex flex-wrap items-end justify-between gap-4">
            <div>
                <h2 class="text-xl font-black text-white">{{ t('admin-builds-title') }}</h2>
                <p class="text-sm text-[var(--noro-muted)]">
                    {{ t('admin-builds-subtitle') }}
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
                        <th>{{ t('admin-builds-col-build') }}</th>
                        <th>Minecraft</th>
                        <th>Modloader</th>
                        <th>{{ t('admin-builds-col-status') }}</th>
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
                                {{ build.published ? t('admin-builds-published') : t('admin-builds-draft') }}
                            </UBadge>
                        </td>
                        <td class="text-right">
                            <AtomButton
                                :to="`/admin/clients/${serverId}?tab=build`"
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
                :title="t('admin-builds-empty-title')"
                :text="t('admin-builds-empty-text')"
            />
        </div>
    </section>
</template>
