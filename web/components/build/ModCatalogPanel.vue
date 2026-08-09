<script setup lang="ts">
/**
 * Вход в каталог модов со страницы сборки.
 *
 * Раньше здесь была модалка с одним полем поиска. Каталог перерос модалку:
 * фасеты, страницы и карточка проекта в неё не помещаются, а прятать их за
 * прокруткой внутри окна — хуже, чем открыть отдельную страницу.
 */
defineProps<{ to: string }>();
</script>

<template>
    <section class="noro-panel overflow-hidden">
        <div class="flex items-center gap-3 border-b border-[var(--noro-border)] px-5 py-4">
            <div class="grid size-10 place-items-center rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] text-[var(--noro-cream)]">
                <UIcon name="i-lucide-library-big" class="size-5" />
            </div>
            <div class="min-w-0">
                <h2 class="font-bold text-[var(--noro-text)]">Mod Catalog</h2>
                <p class="text-[10px] uppercase tracking-wider text-[var(--noro-muted)]">
                    Modrinth &amp; CurseForge
                </p>
            </div>
        </div>

        <div class="grid gap-3 p-4">
            <p class="text-sm text-[var(--noro-muted)]">
                Search both catalogs with filters for this pack's version and loader,
                then install into the build, its game servers, or both.
            </p>
            <AtomButton variant="primary" icon="i-lucide-search" block :to="to">
                Browse mods
            </AtomButton>
        </div>
    </section>
</template>
