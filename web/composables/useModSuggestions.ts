export function useModSuggestions(serverId: Ref<string | undefined>) {
    const auth = useAuth();
    const suggestions = ref<
        Array<{
            id: string;
            title: string;
            icon_url: string | null;
            description: string | null;
            provider: string;
            project_id: string;
            status: string;
            created_at: string;
        }>
    >([]);
    const loading = ref(false);

    async function fetchSuggestions() {
        if (!serverId.value) return;
        loading.value = true;
        try {
            const res = await auth.request<any[]>(
                `/api/admin/mod_suggestions?server_id=${serverId.value}&status=pending`,
            );
            suggestions.value = res || [];
        } catch {
            suggestions.value = [];
        } finally {
            loading.value = false;
        }
    }

    async function approve(id: string) {
        await auth.request(`/api/admin/mod_suggestions/${id}/approve`, { method: "POST" });
        await fetchSuggestions();
    }

    async function accept(
        id: string,
        mode: "optional" | "regular",
        installOnServers = false,
    ) {
        await auth.request(`/api/admin/mod_suggestions/${id}/accept`, {
            method: "POST",
            body: { mode, install_on_servers: installOnServers },
        });
        await fetchSuggestions();
    }

    async function reject(id: string) {
        await auth.request(`/api/admin/mod_suggestions/${id}/reject`, { method: "POST" });
        await fetchSuggestions();
    }

    onMounted(() => fetchSuggestions());
    watch(serverId, () => fetchSuggestions());

    return { suggestions, loading, fetchSuggestions, approve, accept, reject };
}
