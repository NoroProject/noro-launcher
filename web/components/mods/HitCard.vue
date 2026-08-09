<script setup lang="ts">
import type { ModHit } from "~/types/catalog";

const props = defineProps<{
    hit: ModHit;
    selected?: boolean;
}>();

defineEmits<{ select: []; install: [] }>();

const sideLabel = computed(() => {
    const { client_side: client, server_side: server } = props.hit;
    if (client === "unknown") return null;
    const onClient = client !== "unsupported";
    const onServer = server !== "unsupported";
    if (onClient && onServer) return "client + server";
    return onServer ? "server only" : "client only";
});
</script>

<template>
    <article
        class="noro-card grid cursor-pointer gap-3 p-4 transition-colors min-w-0 w-full overflow-hidden shrink-0"
        :class="selected
            ? 'border-[var(--noro-cream)] bg-[var(--noro-panel-2)]'
            : 'hover:border-[var(--noro-blue)]'"
        @click="$emit('select')"
    >
        <div class="flex items-start gap-3">
            <img
                v-if="hit.icon_url"
                :src="hit.icon_url"
                alt=""
                loading="lazy"
                class="size-12 shrink-0 rounded-[var(--noro-r-sm)] object-cover"
            >
            <div
                v-else
                class="grid size-12 shrink-0 place-items-center rounded-[var(--noro-r-sm)] bg-[var(--noro-input)]"
            >
                <UIcon name="i-lucide-box" class="size-5 text-[var(--noro-muted)]" />
            </div>

            <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                    <h3 class="truncate font-bold text-[var(--noro-text)]">{{ hit.title }}</h3>
                    <span
                        class="shrink-0 rounded-[var(--noro-r-sm)] px-2 py-1 text-[10px] font-bold uppercase tracking-wider"
                        :class="hit.provider === 'modrinth'
                            ? 'bg-[color-mix(in_srgb,var(--noro-green)_18%,transparent)] text-[var(--noro-green)]'
                            : 'bg-[color-mix(in_srgb,var(--noro-amber)_18%,transparent)] text-[var(--noro-amber)]'"
                    >{{ hit.provider }}</span>
                </div>
                <p class="mt-1 line-clamp-2 text-sm text-[var(--noro-muted)]">
                    {{ hit.description }}
                </p>
            </div>

            <AtomButton
                variant="primary"
                size="sm"
                icon="i-lucide-download"
                class="shrink-0"
                aria-label="Install"
                @click.stop="$emit('install')"
            />
        </div>

        <div class="flex flex-wrap items-center gap-4 text-xs text-[var(--noro-muted)]">
            <span v-if="hit.author" class="flex items-center gap-1">
                <UIcon name="i-lucide-user" class="size-3" />{{ hit.author }}
            </span>
            <span class="flex items-center gap-1">
                <UIcon name="i-lucide-download" class="size-3" />{{ compactNumber(hit.downloads) }}
            </span>
            <span v-if="hit.updated" class="flex items-center gap-1">
                <UIcon name="i-lucide-clock" class="size-3" />{{ relativeDate(hit.updated) }}
            </span>
            <span v-if="sideLabel" class="flex items-center gap-1">
                <UIcon name="i-lucide-monitor-cog" class="size-3" />{{ sideLabel }}
            </span>
        </div>
    </article>
</template>
