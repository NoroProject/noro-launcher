<script setup lang="ts">
const props = defineProps<{
    form: {
        memoryMin: number;
        memoryMax: number;
        jvmFlags: string;
        showConsole: boolean;
    };
    busy?: string | null;
}>();

defineEmits<{ save: [] }>();
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div class="flex items-center gap-3 border-b border-[var(--noro-border)] px-5 py-4">
            <div class="grid size-10 place-items-center rounded-lg bg-[var(--noro-input)] text-[var(--noro-cream)]">
                <UIcon name="i-lucide-sliders-horizontal" class="size-5" />
            </div>
            <div>
                <h2 class="font-bold text-[var(--noro-text)]">Recommended Client Settings</h2>
                <p class="text-[10px] uppercase tracking-wider text-[var(--noro-muted)]">
                    Defaults sent to launcher for this build
                </p>
            </div>
        </div>

        <div class="grid gap-4 p-5">
            <div class="grid gap-3 sm:grid-cols-2">
                <label>
                    <span class="noro-label-xs">Min Memory</span>
                    <input v-model.number="form.memoryMin" class="noro-input-sm mt-1 w-full" type="number" min="512" step="512" />
                </label>
                <label>
                    <span class="noro-label-xs">Max Memory</span>
                    <input v-model.number="form.memoryMax" class="noro-input-sm mt-1 w-full" type="number" min="512" step="512" />
                </label>
            </div>

            <UCheckbox v-model="form.showConsole" label="Show console window on launch" />

            <label>
                <span class="noro-label-xs">JVM Flags</span>
                <textarea
                    v-model="form.jvmFlags"
                    class="noro-input-sm mt-1 min-h-20 w-full font-mono"
                    placeholder="-XX:+UseG1GC"
                />
            </label>

            <AtomButton
                icon="i-lucide-save"
                variant="primary"
                :loading="busy === 'recommended'"
                block
                @click="$emit('save')"
            >
                Save Recommendations
            </AtomButton>
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
    border: 0 solid transparent;
    border-radius: 8px;
    background: var(--noro-input);
    padding: 0.5rem 0.75rem;
    color: var(--noro-text);
    font-size: 0.8125rem;
}

.noro-input-sm:focus {
    outline: none;
    border-width: 2px;
    border-color: var(--noro-blue);
}
</style>
