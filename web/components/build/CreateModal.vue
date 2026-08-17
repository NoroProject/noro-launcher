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

const { t } = useT();
const isCopy = computed(() => Boolean(props.form.copy_from));
const open = defineModel<boolean>({ required: true });

defineEmits<{
    create: [];
}>();
</script>

<template>
    <AtomModal v-model="open" :title="t('admin-modal-new-build')" :subtitle="t('admin-modal-new-build-sub')">
        <form class="grid gap-4" @submit.prevent="$emit('create')">
            <label v-if="builds.length">
                <span class="noro-label">{{ t('admin-modal-copy-from') }}</span>
                <NoroSelect v-model="form.copy_from">
                    <option value="">{{ t('admin-modal-start-empty') }}</option>
                    <option v-for="build in builds" :key="build.id" :value="build.id">
                        {{ build.version }} — {{ build.mc_version }} {{ build.modloader }}
                    </option>
                </NoroSelect>
                <span class="mt-1 block text-xs text-[var(--noro-muted)]">
                    {{ t('admin-modal-copy-hint') }}
                </span>
            </label>

            <label>
                <span class="noro-label">{{ t('admin-modal-build-version') }}</span>
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
                    v-if="form.modloader !== 'vanilla'"
                    v-model="form.modloader_version"
                    label="Loader version"
                    :options="loaderOptions"
                    :placeholder="t('admin-modal-optional-vanilla')"
                />
            </template>
            <div class="flex justify-end gap-3 pt-2">
                <AtomButton type="button" @click="open = false">{{ t('web-rules-cancel') }}</AtomButton>
                <AtomButton type="submit" variant="primary" :disabled="creating" icon="i-lucide-plus">{{ t('admin-servers-create-btn') }}</AtomButton>
            </div>
        </form>
    </AtomModal>
</template>
