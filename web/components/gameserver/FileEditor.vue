<script setup lang="ts">
import type { GameServer } from "~/types/game-server";

/**
 * Редактор конфига. Кроме сохранения на этот сервер умеет разложить тот же файл
 * на остальные машины сборки — ради этого управление «всеми сразу» и затевалось.
 */
const open = defineModel<boolean>({ required: true });

const props = defineProps<{
    path: string;
    content: string;
    saving?: boolean;
    /** Прочие бэкенды этой сборки: кандидаты на массовое применение. */
    siblings: GameServer[];
}>();

const emit = defineEmits<{
    save: [content: string];
    apply: [content: string, targets: string[]];
}>();

const draft = ref(props.content);
const targets = ref<Set<string>>(new Set());

watch(
    () => [props.content, props.path],
    () => {
        draft.value = props.content;
        targets.value = new Set();
    },
);

function toggle(id: string) {
    const next = new Set(targets.value);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    targets.value = next;
}

const dirty = computed(() => draft.value !== props.content);
</script>

<template>
    <AtomModal v-model="open" title="EDIT FILE" :subtitle="path" wide>
        <div class="grid gap-4">
            <textarea
                v-model="draft"
                class="noro-input h-96 resize-y font-mono text-xs leading-5"
                spellcheck="false"
            />

            <section v-if="siblings.length" class="grid gap-2">
                <span class="noro-label noro-label-inline">Also write to</span>
                <button
                    v-for="server in siblings"
                    :key="server.id"
                    type="button"
                    class="flex items-center gap-3 rounded-[var(--noro-r-sm)] bg-[var(--noro-input)] p-3 text-left"
                    @click="toggle(server.id)"
                >
                    <UIcon
                        :name="targets.has(server.id) ? 'i-lucide-check-square' : 'i-lucide-square'"
                        class="size-4 shrink-0"
                        :class="targets.has(server.id) ? 'text-[var(--noro-cream)]' : 'text-[var(--noro-muted)]'"
                    />
                    <span class="min-w-0 flex-1 truncate text-sm text-[var(--noro-text)]">
                        {{ server.name }}
                    </span>
                    <span
                        class="size-2 shrink-0 rounded-full"
                        :style="{ background: server.live ? 'var(--noro-green)' : 'var(--noro-muted)' }"
                    />
                </button>
            </section>

            <div class="flex flex-wrap gap-2">
                <AtomButton
                    variant="primary"
                    icon="i-lucide-save"
                    :loading="saving"
                    :disabled="!dirty"
                    @click="emit('save', draft)"
                >
                    Save
                </AtomButton>
                <AtomButton
                    v-if="targets.size"
                    variant="secondary"
                    icon="i-lucide-copy"
                    :loading="saving"
                    @click="emit('apply', draft, [...targets])"
                >
                    Save to {{ targets.size + 1 }} servers
                </AtomButton>
                <AtomButton variant="ghost" @click="open = false">Close</AtomButton>
            </div>
        </div>
    </AtomModal>
</template>
