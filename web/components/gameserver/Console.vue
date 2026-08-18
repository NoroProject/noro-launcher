<script setup lang="ts">
const auth = useAuth()
const canCommand = computed(() => auth.hasPermission('noro.admin.wrapper.command'))
/** Живой лог сервера и строка ввода команд. */
const props = defineProps<{ gameServerId: string; enabled: boolean }>();
const emit = defineEmits<{ command: [line: string] }>();

// Не `console`: Vue держит это имя в списке разрешённых глобалей, и в шаблоне
// оно резолвится в глобальный объект, а не в setup-биндинг. Отсюда и падало
// `term.lines.value` — у глобального `console` нет `lines`.
const term = useServerConsole(props.gameServerId);
const input = ref("");
const view = ref<HTMLElement | null>(null);
/** Прилипание к низу: если админ отмотал вверх, дёргать его обратно нельзя. */
const follow = ref(true);

function onScroll() {
    const el = view.value;
    if (!el) return;
    follow.value = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
}

watch(
    () => term.lines.value.length,
    async () => {
        if (!follow.value) return;
        await nextTick();
        const el = view.value;
        if (el) el.scrollTop = el.scrollHeight;
    },
);

function submit() {
    const line = input.value.trim();
    if (!line) return;
    emit("command", line);
    input.value = "";
}

watch(
    () => props.enabled,
    async (on) => {
        if (!on) {
            term.stop();
            return;
        }
        await term.loadBacklog();
        await term.start();
    },
    { immediate: true },
);
</script>

<template>
    <section class="noro-panel flex flex-col h-full min-h-0 gap-3 p-4 overflow-hidden">
        <div class="flex shrink-0 items-center justify-between gap-4">
            <div class="flex items-center gap-2">
                <span class="noro-label noro-label-inline">Console</span>
                <span
                    class="size-2 rounded-full"
                    :style="{ background: term.live.value ? 'var(--noro-green)' : 'var(--noro-muted)' }"
                    :title="term.live.value ? 'streaming' : 'not streaming'"
                />
            </div>
            <div class="flex gap-2">
                <AtomButton variant="ghost" size="sm" icon="i-lucide-eraser" @click="term.clear()">
                    Clear
                </AtomButton>
                <AtomButton
                    variant="ghost"
                    size="sm"
                    :icon="follow ? 'i-lucide-arrow-down-to-line' : 'i-lucide-pause'"
                    :title="follow ? 'Following the tail' : 'Scroll paused'"
                    @click="follow = !follow"
                />
            </div>
        </div>

        <div
            ref="view"
            class="noro-scroll flex-1 min-h-0 overflow-y-auto rounded-[var(--noro-r-sm)] bg-[var(--noro-bg-deep)] p-3 font-mono text-xs leading-5"
            @scroll="onScroll"
        >
            <p
                v-for="(line, i) in term.lines.value"
                :key="i"
                class="whitespace-pre-wrap break-words"
                :class="{
                    'text-[var(--noro-danger)]': line.includes('ERROR') || line.includes('FATAL'),
                    'text-[var(--noro-amber)]': line.includes('WARN'),
                    'text-[var(--noro-muted)]': !line.includes('ERROR') && !line.includes('WARN') && !line.includes('FATAL'),
                }"
            >{{ line }}</p>
            <p v-if="!term.lines.value.length" class="text-[var(--noro-muted)]">
                Nothing yet.
            </p>
        </div>

        <!-- Ввод команды — это выполнение чего угодно на машине, поэтому
             отдельно от права просто читать консоль. -->
        <form v-if="canCommand" class="flex shrink-0 gap-2" @submit.prevent="submit">
            <input
                v-model="input"
                class="noro-input flex-1 font-mono"
                placeholder="say hello"
                autocomplete="off"
                :disabled="!enabled"
            >
            <AtomButton type="submit" variant="primary" icon="i-lucide-corner-down-left" :disabled="!enabled">
                Send
            </AtomButton>
        </form>
    </section>
</template>
