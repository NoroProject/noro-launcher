<script setup lang="ts">
const auth = useAuth()
const canPower = computed(() => auth.hasPermission('noro.admin.wrapper.power'))
import type { PowerAction, WrapperState } from "~/types/wrapper";

/** Состояние машины и кнопки питания. */
const props = defineProps<{
    state: WrapperState | null;
    busy: string | null;
}>();

defineEmits<{ power: [action: PowerAction] }>();

const status = computed(() => props.state?.status);

const phase = computed(() => {
    if (!props.state?.connected) return { label: "wrapper offline", color: "var(--noro-muted)" };
    if (!status.value?.running) return { label: "stopped", color: "var(--noro-danger)" };
    if (!status.value.ready) return { label: "starting", color: "var(--noro-amber)" };
    return { label: "running", color: "var(--noro-green)" };
});

const uptime = computed(() => {
    const total = status.value?.uptime_secs ?? 0;
    if (!total) return "—";
    const hours = Math.floor(total / 3600);
    const minutes = Math.floor((total % 3600) / 60);
    return hours ? `${hours}h ${minutes}m` : `${minutes}m`;
});
</script>

<template>
    <section class="noro-panel grid gap-4 p-5">
        <div class="flex flex-wrap items-center justify-between gap-4">
            <div class="flex items-center gap-3">
                <span class="size-3 shrink-0 rounded-full" :style="{ background: phase.color }" />
                <span class="noro-pixel text-sm uppercase" :style="{ color: phase.color }">
                    {{ phase.label }}
                </span>
            </div>
            <!-- Питание машины — отдельное право: рестарт роняет игроков. -->
            <div v-if="canPower" class="flex flex-wrap gap-2">
                <AtomButton
                    variant="primary"
                    size="sm"
                    icon="i-lucide-play"
                    :disabled="!state?.connected || status?.running"
                    :loading="busy === 'start'"
                    @click="$emit('power', 'start')"
                >
                    Start
                </AtomButton>
                <AtomButton
                    variant="secondary"
                    size="sm"
                    icon="i-lucide-rotate-cw"
                    :disabled="!state?.connected"
                    :loading="busy === 'restart'"
                    @click="$emit('power', 'restart')"
                >
                    Restart
                </AtomButton>
                <AtomButton
                    variant="warning"
                    size="sm"
                    icon="i-lucide-square"
                    :disabled="!state?.connected || !status?.running"
                    :loading="busy === 'stop'"
                    @click="$emit('power', 'stop')"
                >
                    Stop
                </AtomButton>
                <AtomButton
                    variant="danger"
                    size="sm"
                    icon="i-lucide-zap"
                    title="Kill the process without saving"
                    :disabled="!state?.connected || !status?.running"
                    :loading="busy === 'kill'"
                    @click="$emit('power', 'kill')"
                >
                    Kill
                </AtomButton>
            </div>
        </div>

        <UAlert
            v-if="!state?.connected"
            color="warning"
            variant="subtle"
            icon="i-lucide-plug-zap"
            title="No wrapper connected"
            description="Start noro-wrapper on the game machine — it opens the control link itself."
        />

        <dl v-else class="grid gap-4 text-sm sm:grid-cols-4">
            <div>
                <dt class="noro-label">Platform</dt>
                <dd class="text-[var(--noro-text)]">
                    {{ state.info?.platform }} {{ state.info?.mc_version }}
                </dd>
            </div>
            <div>
                <dt class="noro-label">Uptime</dt>
                <dd class="text-[var(--noro-text)]">{{ uptime }}</dd>
            </div>
            <div>
                <dt class="noro-label">Wrapper</dt>
                <dd class="text-[var(--noro-text)]">{{ state.info?.wrapper_version }}</dd>
            </div>
            <div class="min-w-0">
                <dt class="noro-label">Directory</dt>
                <dd class="truncate font-mono text-xs text-[var(--noro-muted)]" :title="state.info?.server_dir">
                    {{ state.info?.server_dir }}
                </dd>
            </div>
        </dl>

        <p v-if="state?.connected && !status?.running && status?.exit_code !== null" class="text-xs text-[var(--noro-muted)]">
            Last exit code: {{ status?.exit_code }}
        </p>
    </section>
</template>
