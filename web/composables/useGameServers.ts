import type { GameServer, GameServerForm } from "~/types/game-server";

/**
 * Игровые сервера сборки: список, регистрация, секреты.
 */
export function useGameServers(serverId: string) {
  const notify = useNotify();
  const api = useApi();
  const base = `/api/admin/servers/${serverId}/game-servers`;

  const items = ref<GameServer[]>([]);
  const pending = ref(false);
  const error = ref<string | null>(null);
  const busyId = ref<string | null>(null);
  const secret = ref<{ name: string; value: string } | null>(null);

  async function load() {
    pending.value = true;
    error.value = null;
    try {
      items.value = await api.requestList<GameServer>(base);
    } catch (err) {
      error.value = "Could not load game servers.";
      console.error(err);
    } finally {
      pending.value = false;
    }
  }

  async function create(form: GameServerForm) {
    const res = await api.request<{ server: GameServer; secret: string }>(base, {
      method: "POST",
      body: form,
    });
    secret.value = { name: res.server.name, value: res.secret };
    await load();
  }

  async function update(id: string, form: GameServerForm) {
    busyId.value = id;
    try {
      await api.request(`${base}/${id}`, { method: "PUT", body: form });
      await load();
      notify.ok();
    } catch (e) {
      notify.fail(e);
    } finally {
      busyId.value = null;
    }
  }

  async function setBulkMaintenance(maintenance: boolean, maintenance_reason?: string, countdown_seconds?: number) {
    pending.value = true;
    try {
      await api.request(`${base}/maintenance`, {
        method: "PUT",
        body: { maintenance, maintenance_reason: maintenance_reason || null, countdown_seconds: countdown_seconds ?? 60 },
      });
      await load();
      notify.ok();
    } catch (e) {
      notify.fail(e);
    } finally {
      pending.value = false;
    }
  }

  async function rotate(item: GameServer) {
    busyId.value = item.id;
    try {
      const res = await api.request<{ secret: string }>(`${base}/${item.id}/token`, {
        method: "POST",
      });
      secret.value = { name: item.name, value: res.secret };
      notify.ok();
    } catch (e) {
      notify.fail(e);
    } finally {
      busyId.value = null;
    }
  }

  async function remove(id: string) {
    busyId.value = id;
    try {
      await api.request(`${base}/${id}`, { method: "DELETE" });
      await load();
      notify.ok('Deleted');
    } catch (e) {
      notify.fail(e);
    } finally {
      busyId.value = null;
    }
  }

  return { items, pending, error, busyId, secret, load, create, update, rotate, remove, setBulkMaintenance };
}
