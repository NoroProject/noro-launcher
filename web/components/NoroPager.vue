<script setup lang="ts">
/**
 * Переключатель страниц под списком.
 *
 * Показывает и общее число найденного: без него «пусто» и «ничего не нашлось по
 * фильтру» выглядят одинаково, а понять, сколько всего записей, неоткуда.
 */
const props = defineProps<{ page: number; pages: number; total: number; perPage: number }>()
const emit = defineEmits<{ go: [page: number] }>()
const { t } = useT()

const from = computed(() => (props.total ? (props.page - 1) * props.perPage + 1 : 0))
const to = computed(() => Math.min(props.page * props.perPage, props.total))
</script>

<template>
    <div
        class="flex flex-wrap items-center justify-between gap-4 border-t border-[var(--noro-border)] px-4 py-3"
    >
        <span class="text-xs text-[var(--noro-muted)]">
            {{ t('paging-range', { from, to, total }) }}
        </span>

        <div v-if="pages > 1" class="flex items-center gap-2">
            <AtomButton
                icon="i-lucide-chevron-left"
                variant="dark"
                :disabled="page <= 1"
                class="!min-h-8 !min-w-8 !px-2"
                @click="emit('go', page - 1)"
            />
            <span class="min-w-24 text-center text-xs text-[var(--noro-muted)]">
                {{ t('paging-page', { page, pages }) }}
            </span>
            <AtomButton
                icon="i-lucide-chevron-right"
                variant="dark"
                :disabled="page >= pages"
                class="!min-h-8 !min-w-8 !px-2"
                @click="emit('go', page + 1)"
            />
        </div>
    </div>
</template>
