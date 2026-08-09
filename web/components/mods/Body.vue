<script setup lang="ts">
import DOMPurify from "dompurify";
import { marked } from "marked";
import type { CatalogProvider } from "~/types/catalog";

/**
 * Описание проекта. Modrinth отдаёт markdown, CurseForge — готовый HTML.
 *
 * И то и другое пишут посторонние люди, поэтому санитайзер обязателен: без него
 * чужое описание получает исполнение скриптов в сессии администратора.
 */
const props = defineProps<{ body: string; provider: CatalogProvider }>();

const html = computed(() => {
    if (!props.body) return "";
    const raw =
        props.provider === "modrinth"
            ? (marked.parse(props.body, { breaks: true, async: false }) as string)
            : props.body;
    return DOMPurify.sanitize(raw);
});
</script>

<template>
    <div v-if="html" class="noro-markdown" v-html="html" />
    <p v-else class="text-sm text-[var(--noro-muted)]">No description provided.</p>
</template>
