<script setup lang="ts">
/** Строка поиска и сортировка. `/` фокусирует поле — руки не уходят с клавиатуры. */
const props = defineProps<{
    modelValue: string;
    sort: string;
    total: number;
    loading?: boolean;
}>();

const emit = defineEmits<{
    "update:modelValue": [value: string];
    "update:sort": [value: string];
}>();

const SORTS = [
    { id: "relevance", label: "Relevance" },
    { id: "downloads", label: "Downloads" },
    { id: "follows", label: "Followers" },
    { id: "updated", label: "Recently updated" },
    { id: "newest", label: "Newest" },
];

/** Прокси для `v-model` селекта: сортировка живёт у родителя. */
const sortModel = computed({
    get: () => props.sort,
    set: (value: string | number) => emit("update:sort", String(value)),
});

const field = ref<HTMLInputElement | null>(null);

function onHotkey(e: KeyboardEvent) {
    const target = e.target as HTMLElement | null;
    const typing = target?.tagName === "INPUT" || target?.tagName === "TEXTAREA";
    if (e.key === "/" && !typing) {
        e.preventDefault();
        field.value?.focus();
    }
}

onMounted(() => window.addEventListener("keydown", onHotkey));
onBeforeUnmount(() => window.removeEventListener("keydown", onHotkey));
</script>

<template>
    <div class="grid gap-3 sm:grid-cols-[1fr_220px] sm:items-center">
        <div class="relative">
            <UIcon
                name="i-lucide-search"
                class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--noro-muted)]"
            />
            <input
                ref="field"
                :value="modelValue"
                class="noro-input !pl-10"
                placeholder="Search mods…  ( / )"
                autocomplete="off"
                @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
            >
            <UIcon
                v-if="loading"
                name="i-lucide-loader-circle"
                class="absolute right-3 top-1/2 size-4 -translate-y-1/2 animate-spin text-[var(--noro-blue)]"
            />
        </div>

        <NoroSelect v-model="sortModel">
            <option v-for="option in SORTS" :key="option.id" :value="option.id">
                {{ option.label }}
            </option>
        </NoroSelect>

        <p class="text-xs text-[var(--noro-muted)] sm:col-span-2">
            {{ props.total ? `${compactNumber(props.total)} result(s)` : "No results yet" }}
        </p>
    </div>
</template>
