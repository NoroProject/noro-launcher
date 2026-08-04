<script setup lang="ts">
import type { BuildCreateForm } from "~/types/server-settings";

defineProps<{
    form: BuildCreateForm;
    minecraft: string[];
    modloaders: string[];
    loaderOptions: string[];
    loading: boolean;
    creating: boolean;
}>();

const open = defineModel<boolean>({ required: true });

defineEmits<{
    create: [];
}>();
</script>

<template>
    <AtomModal v-model="open" title="NEW BUILD" subtitle="Version, Minecraft, and loader metadata">
        <form class="grid gap-4" @submit.prevent="$emit('create')">
            <label>
                <span class="noro-label">Build version</span>
                <input
                    v-model="form.version"
                    class="noro-input"
                    placeholder="1.0.0"
                    required
                >
            </label>
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
            <div class="flex justify-end gap-3 pt-2">
                <AtomButton type="button" @click="open = false">Cancel</AtomButton>
                <AtomButton type="submit" variant="primary" :disabled="creating" icon="i-lucide-plus">Create</AtomButton>
            </div>
        </form>
    </AtomModal>
</template>
