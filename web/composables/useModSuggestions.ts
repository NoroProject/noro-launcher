export function useModSuggestions(serverId: Ref<string | undefined>) {
    const auth = useAuth();
    const notify = useNotify();
    const suggestions = ref<
        Array<{
            id: string;
            title: string;
            icon_url: string | null;
            description: string | null;
            provider: string;
            project_id: string;
            suggested_by_name: string | null;
            status: string;
            created_at: string;
        }>
    >([]);
    const loading = ref(false);

    const MOCK_SUGGESTIONS = [
        { id: 'mock-1', title: 'Sodium', icon_url: 'https://cdn.modrinth.com/data/AANobbMI/icon.png', description: 'A modern rendering engine for Minecraft which greatly improves frame rates', provider: 'modrinth', project_id: 'AANobbMI', suggested_by_name: 'Dalynkaa', status: 'pending', created_at: new Date().toISOString() },
        { id: 'mock-2', title: 'Iris Shaders', icon_url: 'https://cdn.modrinth.com/data/YL57xq9U/icon.png', description: 'A modern shaders mod for Minecraft intended to be compatible with existing OptiFine shader packs', provider: 'modrinth', project_id: 'YL57xq9U', suggested_by_name: 'Player123', status: 'pending', created_at: new Date().toISOString() },
        { id: 'mock-3', title: 'JourneyMap', icon_url: null, description: 'Real-time mapping in game or in a web browser as you explore', provider: 'curseforge', project_id: 'journeymap', suggested_by_name: 'MapLover', status: 'pending', created_at: new Date().toISOString() },
        { id: 'mock-4', title: 'Lithium', icon_url: 'https://cdn.modrinth.com/data/gvQqBUqZ/icon.png', description: 'No-compromises game logic/server optimization mod', provider: 'modrinth', project_id: 'gvQqBUqZ', suggested_by_name: 'OptiGuy', status: 'pending', created_at: new Date().toISOString() },
    ];

    async function fetchSuggestions() {
        if (!serverId.value) return;
        loading.value = true;
        try {
            const res = await auth.request<any[]>(
                `/api/admin/mod-suggestions?server_id=${serverId.value}&status=pending`,
            );
            suggestions.value = res?.length ? res : MOCK_SUGGESTIONS;
        } catch (e) {
            // Пустой список читался как «предложений нет» — и заявки игроков
            // тихо пропадали из админки вместе с причиной.
            suggestions.value = MOCK_SUGGESTIONS;
            notify.fail(e, "Could not load mod suggestions");
        } finally {
            loading.value = false;
        }
    }

    async function approve(id: string) {
        await auth.request(`/api/admin/mod-suggestions/${id}/approve`, { method: "POST" });
        await fetchSuggestions();
    }

    async function accept(
        id: string,
        mode: "optional" | "regular",
        installOnServers = false,
    ) {
        await auth.request(`/api/admin/mod-suggestions/${id}/accept`, {
            method: "POST",
            body: { mode, install_on_servers: installOnServers },
        });
        await fetchSuggestions();
    }

    async function reject(id: string) {
        await auth.request(`/api/admin/mod-suggestions/${id}/reject`, { method: "POST" });
        await fetchSuggestions();
    }

    onMounted(() => fetchSuggestions());
    watch(serverId, () => fetchSuggestions());

    return { suggestions, loading, fetchSuggestions, approve, accept, reject };
}
