<script setup lang="ts">
import type { BuildRow } from "~/types/api";
import type { BuildCreateForm } from "~/types/server-settings";

const props = defineProps<{
    form: BuildCreateForm;
    builds: BuildRow[];
    minecraft: string[];
    modloaders: string[];
    loaderOptions: string[];
    loading: boolean;
    creating: boolean;
}>();

/** Копия наследует загрузчик и версии — спрашивать их второй раз незачем. */
const isCopy = computed(() => Boolean(props.form.copy_from));

const open = defineModel<boolean>({ required: true });

defineEmits<{
    create: [];
}>();
</script>

<template>
    <AtomModal v-model="open" title="NEW BUILD" subtitle="Version, Minecraft, and loader metadata">
        <form class="grid gap-4" @submit.prevent="$emit('create')">
            <label v-if="builds.length">
                <span class="noro-label">Copy from</span>
                <NoroSelect v-model="form.copy_from">
                    <option value="">Start empty</option>
                    <option v-for="build in builds" :key="build.id" :value="build.id">
                        {{ build.version }} — {{ build.mc_version }} {{ build.modloader }}
                    </option>
                </NoroSelect>
                <span class="mt-1 block text-xs text-[var(--noro-muted)]">
                    Carries over every file and setting. Nothing is re-uploaded.
                </span>
            </label>

            <label>
                <span class="noro-label">Build version</span>
                <input
                    v-model="form.version"
                    class="noro-input"
                    placeholder="1.0.0"
                    required
                >
            </label>

            <template v-if="!isCopy">
                <AtomSelect
                    v-model="form.mc_version"
                    label="Minecraft"
                    :options="minecraft"
                    :loading="loading"
                />
                <AtomSelect
                    v-model="form.modloader"
                    label="Modloader"
                    :options="modloaders"
                />
                <AtomSelect
                    v-model="form.modloader_version"
                    label="Loader version"
                    :options="loaderOptions"
                    placeholder="Optional for vanilla"
                />
            </template>
            <div class="flex justify-end gap-3 pt-2">
                <AtomButton type="button" @click="open = false">Cancel</AtomButton>
                <AtomButton type="submit" variant="primary" :disabled="creating" icon="i-lucide-plus">Create</AtomButton>
            </div>
        </form>
    </AtomModal>
</template>
