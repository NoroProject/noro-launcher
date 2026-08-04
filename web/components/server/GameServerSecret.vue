<script setup lang="ts">
/**
 * Секрет агента, показанный один раз.
 *
 * Мастер хранит только sha256, поэтому повторно показать секрет нельзя — можно
 * лишь перевыпустить. Отсюда предупреждение: если его потерять, придётся
 * править конфиг на игровой машине.
 */
const props = defineProps<{ name: string; secret: string }>();
const emit = defineEmits<{ close: [] }>();

const api = useApi();
const copied = ref(false);

const snippet = computed(
    () => `# plugins/NoroAgent/config.yml\nmaster: "${api.masterUrl.value}"\nsecret: "${props.secret}"`,
);

async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
    <AtomModal
        :model-value="true"
        title="AGENT SECRET"
        :subtitle="`For ${name}`"
        @update:model-value="emit('close')"
    >
        <!-- minmax(0,1fr), а не 1fr: иначе длинный секрет растягивает колонку
             грида и содержимое вылезает за панель модалки. -->
        <div class="grid grid-cols-[minmax(0,1fr)] gap-4">
            <UAlert
                color="warning"
                variant="subtle"
                icon="i-lucide-triangle-alert"
                title="Shown once"
                description="The master stores only a hash. If you lose it, issue a new one."
            />

            <div class="min-w-0">
                <span class="noro-label">Secret</span>
                <div class="mt-2 flex items-center gap-2">
                    <code
                        class="min-w-0 flex-1 truncate rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] px-3 py-3 text-xs text-[var(--noro-blue)]"
                    >{{ secret }}</code>
                    <AtomButton
                        :icon="copied ? 'i-lucide-check' : 'i-lucide-copy'"
                        variant="dark"
                        @click="copy(secret)"
                    />
                </div>
            </div>

            <div class="min-w-0">
                <span class="noro-label">Agent config</span>
                <pre
                    class="noro-scroll mt-2 max-w-full overflow-x-auto whitespace-pre rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-bg-deep)] p-4 text-xs leading-5 text-[var(--noro-muted)]"
                >{{ snippet }}</pre>
            </div>

            <AtomButton variant="primary" class="justify-self-end" @click="emit('close')">
                I saved it
            </AtomButton>
        </div>
    </AtomModal>
</template>
