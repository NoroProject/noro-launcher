<script setup lang="ts">
import type { GameServer, GameServerForm } from "~/types/game-server";

const props = defineProps<{ item: GameServer; busy: boolean; manageTo: string }>();
const emit = defineEmits<{
    rotate: [];
    remove: [];
    save: [form: GameServerForm];
}>();

const auth = useAuth();
const notify = useNotify();
const { t } = useT();

const editing = ref(false);
const showingAnnounce = ref(false);
const announceMsg = ref("");
const busyAnnounce = ref(false);

const draft = reactive<GameServerForm>({
    name: props.item.name,
    mc_host: props.item.mc_host,
    mc_port: props.item.mc_port,
    sort_order: props.item.sort_order,
    kind: props.item.kind,
    maintenance: props.item.maintenance,
    maintenance_reason: props.item.maintenance_reason || "",
    countdown_seconds: 60,
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

async function sendServerAnnounce() {
    if (!announceMsg.value.trim()) return;
    busyAnnounce.value = true;
    try {
        await auth.request('/api/admin/game/announce', {
            method: 'POST',
            body: { server_id: props.item.id, message: announceMsg.value.trim() },
        });
        notify.ok();
        announceMsg.value = "";
        showingAnnounce.value = false;
    } catch (e) {
        notify.fail(e);
    } finally {
        busyAnnounce.value = false;
    }
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
                        <UBadge v-if="item.maintenance" color="warning" variant="subtle" size="sm" class="flex items-center gap-1">
                            <UIcon name="i-lucide-wrench" class="size-3" />
                            {{ t('admin-gs-maintenance') }}
                        </UBadge>
                    </div>
                    <div class="truncate text-xs text-[var(--noro-muted)]">
                        {{ item.mc_host || t('admin-gs-no-address') }}:{{ item.mc_port }}
                        <span v-if="item.version"> &middot; {{ item.version }}</span>
                        <span v-if="item.maintenance && item.maintenance_reason" class="text-amber-400"> &middot; {{ item.maintenance_reason }}</span>
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
                    <UTooltip :text="t('admin-moderation-broadcast')">
                        <AtomButton
                            icon="i-lucide-megaphone"
                            variant="ghost"
                            :class="showingAnnounce ? 'text-sky-400' : ''"
                            @click="showingAnnounce = !showingAnnounce"
                        />
                    </UTooltip>
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

        <!-- Inline Announce Row -->
        <div v-if="showingAnnounce && !editing" class="flex flex-wrap items-center gap-2 border-t border-[var(--noro-border)] pt-3">
            <input
                v-model="announceMsg"
                class="noro-input flex-1 min-w-[200px] text-xs"
                :placeholder="t('admin-gs-announce-placeholder')"
                @keydown.enter="sendServerAnnounce"
            >
            <AtomButton
                icon="i-lucide-send"
                variant="primary"
                size="sm"
                :loading="busyAnnounce"
                :disabled="!announceMsg.trim()"
                @click="sendServerAnnounce"
            >
                {{ t('admin-gs-announce-send') }}
            </AtomButton>
        </div>

        <form v-else-if="editing" class="grid gap-3 p-2" @submit.prevent="save">
            <div class="grid gap-3 md:grid-cols-[1fr_1fr_110px_130px_auto_auto]">
                <input v-model="draft.name" class="noro-input" :placeholder="t('admin-roles-name')">
                <input v-model="draft.mc_host" class="noro-input" :placeholder="t('admin-gs-host')">
                <input v-model.number="draft.mc_port" type="number" class="noro-input">
                <NoroSelect v-model="draft.kind">
                    <option value="server">{{ t('admin-gs-backend') }}</option>
                    <option value="proxy">{{ t('admin-gs-proxy') }}</option>
                </NoroSelect>
                <AtomButton type="submit" icon="i-lucide-check" variant="primary">{{ t('web-rules-save') }}</AtomButton>
                <AtomButton icon="i-lucide-x" variant="ghost" @click="editing = false" />
            </div>
            <div class="flex flex-wrap items-center gap-4 border-t border-white/10 pt-3">
                <label class="flex items-center gap-2 cursor-pointer select-none text-sm text-amber-400">
                    <input v-model="draft.maintenance" type="checkbox" class="accent-amber-500">
                    <UIcon name="i-lucide-wrench" class="size-4" />
                    <span>{{ t('admin-gs-maintenance') }}</span>
                </label>
                <input
                    v-if="draft.maintenance"
                    v-model="draft.maintenance_reason"
                    class="noro-input flex-1 min-w-[200px]"
                    :placeholder="t('admin-gs-maintenance-reason')"
                >
                <NoroSelect v-if="draft.maintenance" v-model.number="draft.countdown_seconds" class="w-[180px]" :title="t('admin-gs-maintenance-countdown')">
                    <option :value="0">{{ t('admin-gs-maintenance-countdown-imm') }}</option>
                    <option :value="30">{{ t('admin-gs-maintenance-countdown-30s') }}</option>
                    <option :value="60">{{ t('admin-gs-maintenance-countdown-60s') }}</option>
                    <option :value="300">{{ t('admin-gs-maintenance-countdown-300s') }}</option>
                </NoroSelect>
            </div>
        </form>
    </div>
</template>
