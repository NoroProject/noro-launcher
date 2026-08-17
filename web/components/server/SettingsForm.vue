<script setup lang="ts">
import type { ServerEditForm } from "~/types/server-settings";

defineProps<{
    form: ServerEditForm;
    activeTab: "server" | "client";
    minecraft: string[];
    modloaders: string[];
    loadingMinecraft: boolean;
    saving: boolean;
    deleting: boolean;
}>();

defineEmits<{
    save: [];
    remove: [];
}>();

const { t } = useT();

const copy = computed(() => ({
    server: {
        title: t("admin-set-profile-title"),
        text: t("admin-set-profile-text"),
    },
    client: {
        title: t("admin-set-client-title"),
        text: t("admin-set-client-text"),
    },
}));
</script>

<template>
    <form
        class="noro-panel grid grid-rows-[auto_1fr_auto] overflow-hidden"
        @submit.prevent="$emit('save')"
    >
        <div class="flex items-center gap-4 border-b border-[var(--noro-border)] p-5">
            <div class="grid size-12 place-items-center rounded bg-[var(--noro-input)] text-[var(--noro-blue)]">
                <UIcon name="i-lucide-sliders-horizontal" class="size-6" />
            </div>
            <div class="min-w-0">
                <h2 class="text-lg font-black text-white">
                    {{ copy[activeTab].title }}
                </h2>
                <p class="text-sm text-[var(--noro-muted)]">
                    {{ copy[activeTab].text }}
                </p>
            </div>
        </div>

        <div class="p-5">
            <div v-show="activeTab === 'server'" class="grid gap-5 md:grid-cols-2">
                <label>
                    <span class="noro-label">{{ t('admin-roles-name') }}</span>
                    <input v-model="form.name" class="noro-input" required>
                </label>
                <label>
                    <span class="noro-label">{{ t('admin-roles-order') }}</span>
                    <input v-model.number="form.sort_order" class="noro-input" type="number">
                </label>
                <label class="md:col-span-2">
                    <span class="noro-label">{{ t('admin-blocklist-reason') }}</span>
                    <input v-model="form.description" class="noro-input">
                </label>
                <label class="noro-toggle-row">
                    <UCheckbox v-model="form.active" />
                    <span>
                        <span class="block font-bold text-white">{{ t('admin-set-active') }}</span>
                        <span class="text-sm text-[var(--noro-muted)]">
                            {{ t('admin-set-active-hint') }}
                        </span>
                    </span>
                </label>
                <label class="noro-toggle-row">
                    <UCheckbox v-model="form.limited" />
                    <span>
                        <span class="block font-bold text-white">{{ t('admin-set-limited') }}</span>
                        <span class="text-sm text-[var(--noro-muted)]">
                            {{ t('admin-set-limited-hint') }}
                        </span>
                    </span>
                </label>
            </div>

            <div v-show="activeTab === 'client'" class="grid gap-5 md:grid-cols-2">
                <AtomSelect
                    v-model="form.modloader"
                    label="Modloader"
                    :options="modloaders"
                />
                <AtomSelect
                    v-model="form.mc_version"
                    label="Minecraft"
                    :options="minecraft"
                    :loading="loadingMinecraft"
                />
                <div class="noro-info md:col-span-2">
                    <UIcon name="i-lucide-package-check" class="size-5" />
                    <span>
                        {{ t('admin-set-client-defaults-hint') }}
                    </span>
                </div>
            </div>
        </div>

        <div class="flex flex-wrap gap-3 border-t border-[var(--noro-border)] p-5">
            <AtomButton :loading="saving" type="submit" icon="i-lucide-save" variant="primary">
                {{ t('cabinet-save') }}
            </AtomButton>
            <AtomButton
                :loading="deleting"
                icon="i-lucide-trash-2"
                variant="danger"
                @click="$emit('remove')"
            >
                {{ t('admin-blocklist-act-delete') }}
            </AtomButton>
        </div>
    </form>
</template>
