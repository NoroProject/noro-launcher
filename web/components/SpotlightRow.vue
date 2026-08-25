<script setup lang="ts">
/** Одна строка выдачи спотлайта: аватар или иконка, заголовок и подпись. */
import type { SpotItem } from '~/composables/useSpotlightSources'

defineProps<{ item: SpotItem; active: boolean }>()
</script>

<template>
    <div
        class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-3 text-xs font-bold transition"
        :class="
            active
                ? 'bg-[var(--noro-cream)] text-[var(--noro-on-cream)]'
                : 'text-[var(--noro-text)] hover:bg-[var(--noro-input)]'
        "
    >
        <img
            v-if="item.avatarUrl"
            :src="item.avatarUrl"
            class="size-7 shrink-0 rounded-md border border-[var(--noro-border)] bg-[var(--noro-input)] object-cover"
            alt=""
        />
        <UIcon v-else :name="item.icon" class="size-4 shrink-0" />

        <div class="min-w-0 flex-1">
            <div class="truncate">{{ item.title }}</div>
            <div
                v-if="item.subtitle"
                class="truncate text-[10px] font-normal"
                :class="active ? 'opacity-80' : 'text-[var(--noro-muted)]'"
            >
                {{ item.subtitle }}
            </div>
        </div>
    </div>
</template>
