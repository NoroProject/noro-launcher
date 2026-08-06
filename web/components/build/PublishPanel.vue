<script setup lang="ts">
import type { BuildRow } from "~/types/api";

const props = withDefaults(
    defineProps<{
        build?: BuildRow | null;
        busy?: string | null;
        buildPending?: boolean;
    }>(),
    {},
);
defineEmits<{
    publish: [];
    rebuild: [];
    rebuildClean: [];
    unpublish: [];
    delete: [];
}>();

const isDraft = computed(() => !props.build?.published);
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div
            class="flex items-center gap-3 border-b border-[var(--noro-border)] px-5 py-4"
        >
            <div
                class="flex size-10 items-center justify-center rounded-lg bg-[var(--noro-input)] text-[var(--noro-cream)]"
            >
                <UIcon name="i-lucide-rocket" class="size-6" />
            </div>
            <div>
                <h2 class="font-bold text-[var(--noro-text)]">Release Build</h2>
                <p
                    class="text-xs text-[var(--noro-muted)] uppercase tracking-wider"
                >
                    Bootstrap & Signature
                </p>
            </div>
        </div>

        <div class="p-3">
            <div
                class="mb-2 flex items-center justify-between rounded-lg bg-[var(--noro-input)] p-2"
            >
                <div class="flex items-center gap-3">
                    <div
                        class="size-3 rounded-full animate-pulse"
                        :class="
                            isDraft
                                ? 'bg-[var(--noro-amber)]'
                                : 'bg-[var(--noro-green)]'
                        "
                    />
                    <span
                        class="text-sm font-bold uppercase tracking-widest"
                        :class="
                            isDraft
                                ? 'text-[var(--noro-amber)]'
                                : 'text-[var(--noro-green)]'
                        "
                    >
                        {{ isDraft ? "Draft Mode" : "Live Release" }}
                    </span>
                </div>
                <UTooltip
                    :text="
                        isDraft
                            ? 'This build is not visible to players yet.'
                            : 'This build is currently active in the launcher.'
                    "
                >
                    <UIcon
                        name="i-lucide-info"
                        class="size-4 text-[var(--noro-muted)]"
                    />
                </UTooltip>
            </div>

            <div class="grid gap-1.5">
                <AtomButton
                    v-if="isDraft"
                    icon="i-lucide-zap"
                    variant="primary"
                    block
                    :disabled="busy === 'publish' || buildPending"
                    @click="$emit('publish')"
                >
                    Publish Build
                </AtomButton>

                <AtomButton
                    icon="i-lucide-refresh-cw"
                    variant="warning"
                    block
                    :disabled="busy === 'rebuild'"
                    @click="$emit('rebuild')"
                >
                    Rebuild Version
                </AtomButton>

                <UTooltip
                    text="Discards the Minecraft and loader base, then downloads it again ignoring the cache. Mods and configs are kept. Takes minutes."
                >
                    <AtomButton
                        icon="i-lucide-flame"
                        variant="outline"
                        block
                        :disabled="busy === 'rebuild-clean'"
                        @click="$emit('rebuildClean')"
                    >
                        Rebuild from Scratch
                    </AtomButton>
                </UTooltip>

                <div class="grid grid-cols-2 gap-1.5 mt-0.5">
                    <AtomButton
                        icon="i-lucide-eye-off"
                        variant="dark"
                        block
                        :disabled="isDraft || busy === 'unpublish'"
                        @click="$emit('unpublish')"
                    >
                        Revert to Draft
                    </AtomButton>
                    <AtomButton
                        icon="i-lucide-trash-2"
                        variant="danger"
                        block
                        :disabled="busy === 'delete'"
                        @click="$emit('delete')"
                    >
                        Delete Build
                    </AtomButton>
                </div>
            </div>
        </div>
    </section>
</template>
