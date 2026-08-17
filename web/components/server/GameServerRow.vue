<script setup lang="ts">
import type { GameServer, GameServerForm } from "~/types/game-server";

const props = defineProps<{ item: GameServer; busy: boolean; manageTo: string }>();
const emit = defineEmits<{
    rotate: [];
    remove: [];
    save: [form: GameServerForm];
}>();

const { t } = useT();
const editing = ref(false);
const draft = reactive<GameServerForm>({
    name: props.item.name,
    mc_host: props.item.mc_host,
    mc_port: props.item.mc_port,
    sort_order: props.item.sort_order,
    kind: props.item.kind,
});

const isProxy = computed(() => props.item.kind === "proxy");

const lastSeen = computed(() => {
    if (!props.item.last_seen_at) return t('admin-gs-never-seen');
    const mins = Math.floor((Date.now() - Date.parse(props.item.last_seen_at)) / 60000);
    if (mins < 1) return t('admin-gs-just-now');
    if (mins < 60) return t('admin-gs-ago', { time: `${mins}m` });
    return t('admin-gs-ago', { time: `${Math.floor(mins / 60)}h` });
});

function save() {
    emit("save", { ...draft });
    editing.value = false;
}
</script>

<template>
    <div class="grid gap-3 px-5 py-4 transition hover:bg-white/5">
        <div v-if="!editing" class="flex flex-wrap items-center justify-between gap-4">
            <div class="flex min-w-0 items-center gap-3">
                <div class="relative shrink-0">
                    <img
                        v-if="item.icon_url"
                        :src="item.icon_url"
                        :alt="item.name"
                        class="size-10 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] object-cover"
                    >
                    <div
                        v-else
                        class="grid size-10 place-items-center rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-muted)]"
                    >
                        <UIcon name="i-lucide-server" class="size-5" />
                    </div>
                    <span
                        class="absolute -bottom-0.5 -right-0.5 size-3 rounded-full border-2 border-[var(--noro-panel)]"
                        :class="item.live ? 'bg-[var(--noro-success)]' : 'bg-[var(--noro-muted)]'"
                        :title="item.live ? t('web-rules-status-online') : t('web-rules-status-offline')"
                    />
                </div>
                <div class="min-w-0">
                    <div class="flex items-center gap-2">
                        <span class="truncate text-sm font-bold text-white">{{ item.name }}</span>
                        <UBadge v-if="isProxy" color="info" variant="subtle" size="sm">{{ t('admin-gs-proxy') }}</UBadge>
                    </div>
                    <div class="truncate text-xs text-[var(--noro-muted)]">
                        {{ item.mc_host || t('admin-gs-no-address') }}:{{ item.mc_port }}
                        <span v-if="item.version"> &middot; {{ item.version }}</span>
                    </div>
                </div>
            </div>
            <div class="flex items-center gap-4">
                <div class="text-right">
                    <div class="text-sm font-bold text-white">
                        {{ item.live ? `${item.online}/${item.max_online}` : "&mdash;" }}
                    </div>
                    <div class="text-xs text-[var(--noro-muted)]">{{ lastSeen }}</div>
                </div>
                <div class="flex items-center gap-1">
                    <UTooltip v-if="!isProxy" :text="t('admin-gs-control-tooltip')">
                        <AtomButton icon="i-lucide-sliders-horizontal" variant="ghost" :to="manageTo" />
                    </UTooltip>
                    <UTooltip :text="t('profile-edit')">
                        <AtomButton icon="i-lucide-pencil" variant="ghost" @click="editing = true" />
                    </UTooltip>
                    <UTooltip :text="t('admin-gs-rotate-tooltip')">
                        <AtomButton
                            icon="i-lucide-key-round"
                            variant="ghost"
                            :loading="busy"
                            @click="emit('rotate')"
                        />
                    </UTooltip>
                    <UTooltip :text="t('admin-blocklist-act-delete')">
                        <AtomButton
                            icon="i-lucide-trash-2"
                            variant="ghost"
                            :loading="busy"
                            @click="emit('remove')"
                        />
                    </UTooltip>
                </div>
            </div>
        </div>

        <form v-else class="grid gap-3 md:grid-cols-[1fr_1fr_110px_130px_auto_auto]" @submit.prevent="save">
            <input v-model="draft.name" class="noro-input" :placeholder="t('admin-roles-name')">
            <input v-model="draft.mc_host" class="noro-input" :placeholder="t('admin-gs-host')">
            <input v-model.number="draft.mc_port" type="number" class="noro-input">
            <NoroSelect v-model="draft.kind">
                <option value="server">{{ t('admin-gs-backend') }}</option>
                <option value="proxy">{{ t('admin-gs-proxy') }}</option>
            </NoroSelect>
            <AtomButton type="submit" icon="i-lucide-check" variant="primary">{{ t('web-rules-save') }}</AtomButton>
            <AtomButton icon="i-lucide-x" variant="ghost" @click="editing = false" />
        </form>
    </div>
</template>
