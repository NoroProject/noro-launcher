<script setup lang="ts">
import type { ServerRow } from "~/types/api";

const auth = useAuth();

const notify = useNotify()
const { minecraft, loader, loading, loadMinecraft, loadLoader } =
    useVersionOptions();
await auth.loadMe();
const modloaders = ["vanilla", "fabric", "neoforge", "forge", "quilt"];

const {
    data: servers,
    refresh,
    pending,
    error,
} = await useAsyncData(
    "admin-servers",
    () => auth.request<ServerRow[]>("/api/admin/servers"),
    { default: () => [] },
);

const form = reactive({
    name: "",
    modloader: "fabric",
    mc_version: "1.21.1",
    modloader_version: "",
    create_build: true,
    build_version: "1.0.0",
});
const creating = ref(false);
const showCreate = ref(false);
const loaderOptions = computed(() => loader.value[form.modloader] || []);

onMounted(loadMinecraft);

watch(
    () => [form.modloader, form.mc_version],
    ([kind, mc]) => {
        loadLoader(kind, mc).catch(() => {});
    },
    { immediate: true },
);

async function createServer() {
    creating.value = true;
    try {
        const body = {
            ...form,
            build_version: form.create_build ? form.build_version : undefined,
            modloader_version:
                form.create_build && form.modloader !== "vanilla"
                    ? form.modloader_version || null
                    : undefined,
        };
        await auth.request("/api/admin/servers", { method: "POST", body });
        Object.assign(form, {
            name: "",
            modloader: "fabric",
            mc_version: "1.21.1",
            modloader_version: "",
            create_build: true,
            build_version: "1.0.0",
        });
        await refresh();
        showCreate.value = false;
      notify.ok()
    } catch (e) {
      notify.fail(e)
    } finally {
        creating.value = false;
    }
}
</script>

<template>
    <NoroShell
        title="SERVERS"
        subtitle="Server profiles, order, and launch metadata"
    >
        <template #actions>
            <AtomButton
              icon="i-lucide-refresh-cw"
              variant="dark"
              :loading="pending"
              @click="refresh()"
            >
              Refresh
            </AtomButton>
            <AtomButton
              icon="i-lucide-plus"
              variant="primary"
              @click="showCreate = true"
            >
              New server
            </AtomButton>
        </template>

        <UAlert
            v-if="error"
            class="mb-5"
            color="error"
            variant="subtle"
            icon="i-lucide-circle-alert"
            :description="humanError(error)"
        />

        <section class="noro-panel overflow-hidden">
            <table v-if="servers?.length" class="noro-table w-full">
                <thead>
                    <tr>
                        <th class="px-5 py-3 text-left">Server</th>
                        <th
                            class="px-5 py-3 text-left text-xs uppercase text-[var(--noro-muted)]"
                        >
                            Stack
                        </th>
                        <th
                            class="px-5 py-3 text-left text-xs uppercase text-[var(--noro-muted)]"
                        >
                            Status
                        </th>
                        <th class="px-5 py-3" />
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="server in servers"
                        :key="server.id"
                        class="border-t border-[var(--noro-border)] hover:bg-white/5"
                    >
                        <td class="px-5 py-4">
                            <div class="flex items-center gap-3">
                                <img
                                    v-if="server.icon_url"
                                    :src="server.icon_url"
                                    :alt="server.name"
                                    class="size-10 shrink-0 rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] object-cover"
                                >
                                <div
                                    v-else
                                    class="grid size-10 shrink-0 place-items-center rounded-lg border border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-muted)]"
                                >
                                    <UIcon name="i-lucide-box" class="size-5" />
                                </div>
                                <div class="min-w-0">
                                    <div class="font-bold text-[var(--noro-cream)] truncate">
                                        {{ server.name }}
                                    </div>
                                    <div class="text-xs text-[var(--noro-muted)] truncate">
                                        {{ server.description || "No description" }}
                                    </div>
                                </div>
                            </div>
                        </td>
                        <td class="px-5 py-4">
                            <div
                                class="flex items-center gap-2 text-sm text-[var(--noro-text)]"
                            >
                                <span class="capitalize">{{
                                    server.modloader
                                }}</span>
                                <span class="text-[var(--noro-muted)]">·</span>
                                <span>{{ server.mc_version }}</span>
                            </div>
                        </td>
                        <td class="px-5 py-4">
                            <div class="flex items-center gap-2">
                                <UBadge
                                    :color="
                                        server.active ? 'success' : 'neutral'
                                    "
                                    variant="subtle"
                                    >{{
                                        server.active ? "active" : "off"
                                    }}</UBadge
                                >
                                <UBadge
                                    v-if="server.limited"
                                    color="warning"
                                    variant="subtle"
                                    >limited</UBadge
                                >
                            </div>
                        </td>
                        <td class="px-5 py-4 text-right">
                            <AtomButton
                              variant="dark"
                              icon="i-lucide-settings-2"
                              :to="`/admin/clients/${server.id}`"
                              class="!min-h-9 !min-w-9 !px-2"
                            >

                            </AtomButton>
                        </td>
                    </tr>
                </tbody>
            </table>
            <div v-else class="p-12">
                <EmptyState
                    icon="i-lucide-server"
                    title="No servers yet"
                    text="Create your first server profile to get started."
                />
            </div>
        </section>

        <AtomModal
            v-model="showCreate"
            title="NEW SERVER"
            subtitle="Create a server and its first build profile"
        >
            <form class="grid gap-5" @submit.prevent="createServer">
                <div class="grid gap-3">
                    <label
                        ><span class="noro-label">Server Name</span
                        ><input
                            v-model="form.name"
                            class="noro-input"
                            placeholder="My Survival Server"
                            required
                    /></label>
                    <p class="text-xs text-[var(--noro-muted)]">
                        Addresses are set per game server once the pack exists.
                    </p>
                </div>

                <div class="rounded-lg bg-black/20 p-4">
                    <div class="mb-4 flex items-center justify-between">
                        <h3
                            class="text-xs font-bold uppercase tracking-wider text-[var(--noro-muted)]"
                        >
                            Initial Build Configuration
                        </h3>
                        <UCheckbox v-model="form.create_build" />
                    </div>

                    <div v-if="form.create_build" class="grid gap-3">
                        <label
                            ><span class="noro-label">Build Version</span
                            ><input
                                v-model="form.build_version"
                                class="noro-input"
                                placeholder="1.0.0"
                        /></label>
                        <div class="grid grid-cols-2 gap-3">
                            <AtomSelect
                                v-model="form.mc_version"
                                label="Minecraft"
                                :options="minecraft"
                                :loading="loading"
                            />
                            <AtomSelect
                                v-model="form.modloader"
                                label="Modloader"
                                :options="modloaders"
                            />
                        </div>
                        <AtomSelect
                            v-if="form.modloader !== 'vanilla'"
                            v-model="form.modloader_version"
                            label="Loader Version"
                            :options="loaderOptions"
                            placeholder="Select version..."
                            required
                        />
                    </div>
                    <div
                        v-else
                        class="py-2 text-center text-xs italic text-[var(--noro-muted)]"
                    >
                        You can create builds later in the server settings.
                    </div>
                </div>

                <!-- AtomButton, а не UButton: тема Nuxt UI не знает про
                     кремовый токен и красила кнопку в серый — она читалась
                     как выключенная. -->
                <div class="flex justify-end gap-3 pt-2">
                    <AtomButton variant="ghost" @click="showCreate = false">
                        Cancel
                    </AtomButton>
                    <AtomButton
                      :loading="creating"
                      type="submit"
                      variant="primary"
                      icon="i-lucide-plus"
                    >
                      Create Server
                    </AtomButton>
                </div>
            </form>
        </AtomModal>
    </NoroShell>
</template>
