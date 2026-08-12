<script setup lang="ts">
import type { InstalledModInfo, ModVersion } from "~/types/catalog";

const props = defineProps<{
    version: ModVersion;
    installedMod?: InstalledModInfo | null;
}>();
defineEmits<{ install: [] }>();

const requires = computed(
    () => props.version.dependencies.filter((d) => d.kind === "required").length,
);

const isCurrent = computed(() => {
    if (!props.installedMod?.version) return false;
    const v1 = props.version.version_number.trim().toLowerCase();
    const v2 = props.installedMod.version.trim().toLowerCase();
    return v1 === v2 || v1.includes(v2) || v2.includes(v1);
});
</script>

<template>
    <div class="flex items-center gap-3 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-3 min-w-0 w-full overflow-hidden shrink-0">
        <span
            class="size-2 shrink-0 rounded-full"
            :style="{ background: channelColor(version.channel) }"
            :title="version.channel"
        />
        <div class="min-w-0 flex-1">
            <div class="truncate text-xs md:text-sm font-bold text-[var(--noro-text)]" :title="version.name">
                {{ version.name }}
            </div>
            <div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[11px] text-[var(--noro-muted)]">
                <span>{{ version.game_versions.slice(0, 2).join(", ") || "any" }}</span>
                <span v-if="version.loaders.length" class="uppercase font-mono text-[10px] opacity-80">{{ version.loaders.join(", ") }}</span>
                <span>{{ compactBytes(version.size) }}</span>
                <span v-if="version.published">{{ relativeDate(version.published) }}</span>
                <span v-if="requires" class="text-[var(--noro-amber)] font-medium">
                    {{ requires }} dep
                </span>
            </div>
        </div>
        <UBadge
            v-if="isCurrent"
            color="success"
            variant="subtle"
            size="sm"
            class="shrink-0"
        >
            Current
        </UBadge>
        <AtomButton
            v-else
            :variant="installedMod ? 'secondary' : 'primary'"
            size="sm"
            :icon="installedMod ? 'i-lucide-refresh-cw' : 'i-lucide-download'"
            :disabled="!version.downloadable"
            :title="installedMod ? 'Update or Reinstall to this version' : 'Install this version'"
            class="shrink-0"
            @click="$emit('install')"
        >
            {{ installedMod ? 'Update' : '' }}
        </AtomButton>
    </div>
</template>
