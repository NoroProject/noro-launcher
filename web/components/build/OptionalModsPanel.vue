<script setup lang="ts">
import type { OptionalMod } from "~/types/api";

const props = withDefaults(
    defineProps<{
        optionalMods?: OptionalMod[] | null;
        busy?: string | null;
    }>(),
    {},
);
defineEmits<{ save: []; delete: [index: number] }>();

function setFiles(mod: OptionalMod, value: string) {
    mod.files = value
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
}
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <!-- Header -->
        <div
            class="flex items-center justify-between border-b border-[var(--noro-border)] px-5 py-4"
        >
            <div class="flex items-center gap-3">
                <div
                    class="flex size-10 items-center justify-center rounded-lg bg-black/20 text-[var(--noro-magenta)]"
                >
                    <UIcon name="i-lucide-package-plus" class="size-6" />
                </div>
                <div>
                    <h2 class="font-bold text-[var(--noro-text)]">
                        Optional Mods
                    </h2>
                    <p
                        class="text-xs text-[var(--noro-muted)] uppercase tracking-wider"
                    >
                        {{ optionalMods?.length || 0 }} mods configured
                    </p>
                </div>
            </div>
        </div>

        <div class="p-5">
            <div
                v-if="!optionalMods?.length"
                class="flex flex-col items-center justify-center py-6 text-center"
            >
                <div class="mx-auto mb-2 grid size-10 place-items-center rounded-lg bg-[var(--noro-input)]">
                  <UIcon name="i-lucide-box-select" class="size-5 text-[var(--noro-cream)]" />
                </div>
                <div class="text-sm font-bold text-[var(--noro-text)]">No optional mods</div>
                <p class="mt-1 max-w-[220px] text-[10px] text-[var(--noro-muted)]">Players can enable optional components.</p>
            </div>

            <div v-else class="grid gap-5">
                <div
                    v-for="(mod, index) in optionalMods"
                    :key="index"
                    class="group relative flex flex-col rounded-xl border border-[var(--noro-border)] bg-black/20 p-5 transition-all hover:border-[var(--noro-magenta)]/50 hover:bg-black/30"
                >
                    <!-- Floating Actions -->
                    <div class="absolute right-4 top-4 flex gap-2">
                        <AtomButton variant="danger"
                            icon="i-lucide-trash-2"
                            size="sm"
                            class="opacity-0 transition-opacity group-hover:opacity-100"
                            @click="$emit('delete', index)"
                        />
                    </div>

                    <div class="grid gap-4 md:grid-cols-3">
                        <!-- Primary Info -->
                        <div class="md:col-span-2 grid gap-4">
                            <div class="grid gap-4 sm:grid-cols-2">
                                <label class="block">
                                    <span class="noro-label-xs">Display Name</span>
                                    <input
                                        v-model="mod.name"
                                        class="noro-input-sm mt-1 w-full"
                                        placeholder="e.g. Sodium Extras"
                                    />
                                </label>
                                <label class="block">
                                    <span class="noro-label-xs">Category</span>
                                    <input
                                        v-model="mod.category"
                                        class="noro-input-sm mt-1 w-full"
                                        placeholder="e.g. Performance"
                                    />
                                </label>
                                <label class="block">
                                    <span class="noro-label-xs">Author</span>
                                    <input
                                        v-model="mod.author"
                                        class="noro-input-sm mt-1 w-full"
                                        placeholder="e.g. jellysquid3"
                                    />
                                </label>
                                <label class="block">
                                    <span class="noro-label-xs">Icon URL</span>
                                    <input
                                        v-model="mod.icon_url"
                                        class="noro-input-sm mt-1 w-full font-mono"
                                        placeholder="https://cdn.modrinth.com/..."
                                    />
                                </label>
                            </div>

                            <label class="block">
                                <span class="noro-label-xs">Files (CSV)</span>
                                <input
                                    class="noro-input-sm mt-1 w-full font-mono"
                                    :value="mod.files.join(', ')"
                                    placeholder="mods/mod-a.jar, config/mod-a.toml"
                                    @input="
                                        setFiles(
                                            mod,
                                            ($event.target as HTMLInputElement)
                                                .value,
                                        )
                                    "
                                />
                            </label>
                        </div>

                        <!-- Config & Meta -->
                        <div
                            class="flex flex-col gap-4 rounded-lg bg-black/20 p-4"
                        >
                            <span
                                class="noro-label-xs uppercase tracking-widest text-[var(--noro-muted)]"
                                >Behavior</span
                            >
                            <div class="grid gap-3">
                                <UCheckbox
                                    v-model="mod.enabled_by_default"
                                    label="Enabled by default"
                                    class="text-xs"
                                />
                                <UCheckbox
                                    v-model="mod.visible"
                                    label="Visible in UI"
                                    class="text-xs"
                                />
                                <UCheckbox
                                    v-model="mod.limited"
                                    label="Restricted access"
                                    class="text-xs"
                                />
                            </div>
                        </div>
                    </div>

                    <!-- Description -->
                    <div
                        class="mt-4 border-t border-[var(--noro-border)]/30 pt-4"
                    >
                        <label class="block">
                            <span class="noro-label-xs">Description</span>
                            <textarea
                                v-model="mod.description"
                                class="noro-input-sm mt-1 w-full min-h-[60px] resize-none"
                                placeholder="What does this mod do? This will be shown in the launcher."
                            />
                        </label>
                    </div>
                </div>
            </div>

            <!-- Footer Action -->
            <div
                v-if="optionalMods?.length"
                class="mt-8 flex items-center justify-between rounded-lg bg-[var(--noro-magenta)]/5 p-4 border border-[var(--noro-magenta)]/20"
            >
                <p class="text-sm text-[var(--noro-muted)] italic">
                    Remember to save your changes to apply them to the build
                    manifest.
                </p>
                <AtomButton variant="primary"
                    :loading="busy === 'optional'"
                    icon="i-lucide-save"
                    size="md"
                    @click="$emit('save')"
                >
                    Save Optional Mods
                </AtomButton>
            </div>
        </div>
    </section>
</template>

<style scoped>
.noro-label-xs {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--noro-muted);
}

.noro-input-sm {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--noro-border);
    border-radius: 6px;
    color: var(--noro-text);
    font-size: 0.8125rem;
    padding: 0.5rem 0.75rem;
    transition: all 0.2s;
}

.noro-input-sm:focus {
    outline: none;
    border-color: var(--noro-magenta);
    box-shadow: 0 0 0 2px rgba(232, 90, 165, 0.1);
}

textarea.noro-input-sm {
    font-family: inherit;
    line-height: 1.5;
}
</style>
