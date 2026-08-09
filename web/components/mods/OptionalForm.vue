<script setup lang="ts">
import type { OptionalDraft } from "~/types/catalog";

/**
 * Карточка опционального мода. Имя и категорию задаёт человек: в лаунчере это
 * подпись в списке переключателей, а не имя файла с Modrinth.
 */
const draft = defineModel<OptionalDraft>({ required: true });
</script>

<template>
    <div class="grid gap-3 border-l-2 border-[var(--noro-cream)]/30 pl-4">
        <div class="grid gap-3 sm:grid-cols-[1fr_160px]">
            <label class="block">
                <span class="noro-label">Display name</span>
                <input v-model="draft.name" class="noro-input" placeholder="Sodium">
            </label>
            <label class="block">
                <span class="noro-label">Category</span>
                <input v-model="draft.category" class="noro-input" placeholder="Performance">
            </label>
        </div>
        <label class="block">
            <span class="noro-label">Description</span>
            <textarea
                v-model="draft.description"
                class="noro-input resize-none"
                rows="2"
                placeholder="What the player gets by turning this on"
            />
        </label>
        <div class="flex flex-wrap gap-4 text-sm text-[var(--noro-text)]">
            <label class="flex items-center gap-2">
                <input v-model="draft.enabled_by_default" type="checkbox"> Enabled by default
            </label>
            <label class="flex items-center gap-2">
                <input v-model="draft.visible" type="checkbox"> Visible
            </label>
            <label class="flex items-center gap-2">
                <input v-model="draft.limited" type="checkbox"> Restricted
            </label>
        </div>
    </div>
</template>
