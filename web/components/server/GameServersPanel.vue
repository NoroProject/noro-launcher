<script setup lang="ts">
import type { GameServerForm } from "~/types/game-server";

const props = defineProps<{ serverId: string }>();

const gs = useGameServers(props.serverId);
const { t } = useT();
const adding = ref(false);
function blank(): GameServerForm {
    return { name: "", mc_host: "", mc_port: 25565, sort_order: 0, kind: "server" };
}

const form = reactive<GameServerForm>(blank());

const totalOnline = computed(() => {
    const live = gs.items.value.filter(i => i.live);
    const backends = live.filter(i => i.kind !== "proxy");
    return (backends.length ? backends : live).reduce((sum, i) => sum + i.online, 0);
});

async function submit() {
    if (!form.name.trim()) return;
    await gs.create({ ...form });
    Object.assign(form, blank());
    adding.value = false;
}

onMounted(gs.load);
</script>

<template>
    <section class="grid gap-4">
        <div class="flex flex-wrap items-end justify-between gap-4">
            <div>
                <h2 class="text-xl font-black text-white">{{ t('admin-gs-title') }}</h2>
                <p class="text-sm text-[var(--noro-muted)]">
                    {{ t('admin-gs-subtitle') }}
                </p>
            </div>
            <div class="flex items-center gap-3">
                <span v-if="gs.items.value.length" class="noro-label noro-label-inline">
                    {{ totalOnline }} online
                </span>
                <AtomButton icon="i-lucide-plus" variant="primary" @click="adding = !adding">
                    {{ t('admin-gs-add') }}
                </AtomButton>
            </div>
        </div>

        <UAlert
            v-if="gs.error.value"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="gs.error.value"
        />

        <form v-if="adding" class="noro-panel grid gap-3 p-5 md:grid-cols-[1fr_1fr_110px_130px_auto]" @submit.prevent="submit">
            <label class="min-w-0">
                <span class="noro-label">{{ t('admin-roles-name') }}</span>
                <input v-model="form.name" class="noro-input mt-2" placeholder="Survival" autocomplete="off">
            </label>
            <label class="min-w-0">
                <span class="noro-label">{{ t('admin-gs-host') }}</span>
                <input v-model="form.mc_host" class="noro-input mt-2" placeholder="10.0.0.5" autocomplete="off">
            </label>
            <label class="min-w-0">
                <span class="noro-label">{{ t('admin-gs-port') }}</span>
                <input v-model.number="form.mc_port" type="number" class="noro-input mt-2">
            </label>
            <label class="min-w-0">
                <span class="noro-label">{{ t('admin-punish-kind') }}</span>
                <NoroSelect v-model="form.kind" class="mt-2">
                    <option value="server">{{ t('admin-gs-backend') }}</option>
                    <option value="proxy">{{ t('admin-gs-proxy') }}</option>
                </NoroSelect>
            </label>
            <AtomButton type="submit" icon="i-lucide-check" variant="primary" class="self-end">
                {{ t('admin-gs-create') }}
            </AtomButton>
        </form>

        <div class="noro-panel overflow-hidden">
            <div v-if="gs.items.value.length" class="divide-y divide-[var(--noro-border)]">
                <ServerGameServerRow
                    v-for="item in gs.items.value"
                    :key="item.id"
                    :item="item"
                    :busy="gs.busyId.value === item.id"
                    :manage-to="`/admin/clients/${serverId}/game/${item.id}`"
                    @rotate="gs.rotate(item)"
                    @remove="gs.remove(item.id)"
                    @save="next => gs.update(item.id, next)"
                />
            </div>
            <EmptyState
                v-else
                icon="i-lucide-server-cog"
                :title="t('admin-gs-empty-title')"
                :text="t('admin-gs-empty-text')"
            />
        </div>

        <ServerGameServerSecret
            v-if="gs.secret.value"
            :name="gs.secret.value.name"
            :secret="gs.secret.value.value"
            @close="gs.secret.value = null"
        />
    </section>
</template>
