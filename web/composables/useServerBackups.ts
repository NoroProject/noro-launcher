import type { ServerBackup, ServerBackupsResponse } from "~/types/wrapper";

/**
 * Бэкапы серверной директории игрового сервера.
 *
 * Создание снимков, восстановление и удаление архивов через ServerWrapper.
 */
export function useServerBackups(gameServerId: string) {
  const auth = useAuth();
  const notify = useNotify();
  const base = `/api/admin/game-servers/${gameServerId}/backups`;

  const backups = ref<ServerBackup[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function refresh() {
    loading.value = true;
    error.value = null;
    try {
      const res = await auth.request<ServerBackupsResponse>(base);
      backups.value = res?.backups ?? [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      backups.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function create(name: string) {
    loading.value = true;
    try {
      await auth.request(base, {
        method: "POST",
        body: { name },
      });
      notify.success("Snapshot created");
      await refresh();
    } catch (e) {
      notify.error(e instanceof Error ? e.message : "Failed to create snapshot");
    } finally {
      loading.value = false;
    }
  }

  async function restore(name: string) {
    loading.value = true;
    try {
      await auth.request(`${base}/${encodeURIComponent(name)}/restore`, {
        method: "POST",
      });
      notify.success("Snapshot restored. Restart the server to apply changes.");
      await refresh();
    } catch (e) {
      notify.error(e instanceof Error ? e.message : "Failed to restore snapshot");
    } finally {
      loading.value = false;
    }
  }

  async function remove(name: string) {
    loading.value = true;
    try {
      await auth.request(`${base}/${encodeURIComponent(name)}`, {
        method: "DELETE",
      });
      notify.success("Snapshot deleted");
      await refresh();
    } catch (e) {
      notify.error(e instanceof Error ? e.message : "Failed to delete snapshot");
    } finally {
      loading.value = false;
    }
  }

  return {
    backups,
    loading,
    error,
    refresh,
    create,
    restore,
    remove,
  };
}
