<script setup lang="ts">
import type {
    ServerEditForm,
    ServerSettingsTab,
} from "~/types/server-settings";

defineProps<{
    form: ServerEditForm;
    activeTab: ServerSettingsTab;
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

const copy = {
    server: {
        title: "Profile",
        text: "How this pack is named and where it shows up.",
    },
    client: {
        title: "Client defaults",
        text: "Version and loader new builds inherit from this pack.",
    },
};
</script>

<template>
    <!--
      Вкладки переключаются через v-show, а не наложением в одной ячейке грида.
      Наложение делало панель высотой с самую длинную вкладку — на короткой
      оставалась дыра, — а z-10 у активной перекрывал липкую шапку страницы.
    -->
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
                    <span class="noro-label">Name</span>
                    <input v-model="form.name" class="noro-input" required>
                </label>
                <label>
                    <span class="noro-label">Launcher order</span>
                    <input v-model.number="form.sort_order" class="noro-input" type="number">
                </label>
                <label class="md:col-span-2">
                    <span class="noro-label">Description</span>
                    <input v-model="form.description" class="noro-input">
                </label>
                <label class="noro-toggle-row">
                    <UCheckbox v-model="form.active" />
                    <span>
                        <span class="block font-bold text-white">Active</span>
                        <span class="text-sm text-[var(--noro-muted)]">
                            Show this pack in launcher lists.
                        </span>
                    </span>
                </label>
                <label class="noro-toggle-row">
                    <UCheckbox v-model="form.limited" />
                    <span>
                        <span class="block font-bold text-white">Limited access</span>
                        <span class="text-sm text-[var(--noro-muted)]">
                            Require role access before players can join.
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
                        Builds inherit these defaults when a new build is
                        created from this pack. Server addresses live on the game
                        servers below.
                    </span>
                </div>
            </div>
        </div>

        <div class="flex flex-wrap gap-3 border-t border-[var(--noro-border)] p-5">
            <AtomButton :loading="saving" type="submit" icon="i-lucide-save" variant="primary">
                Save
            </AtomButton>
            <AtomButton
                :loading="deleting"
                icon="i-lucide-trash-2"
                variant="danger"
                @click="$emit('remove')"
            >
                Delete
            </AtomButton>
        </div>
    </form>
</template>
