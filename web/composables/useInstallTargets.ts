import type { BuildRow, ServerRow } from "~/types/api";
import type { GameServer } from "~/types/game-server";
import type { InstallTarget } from "~/types/catalog";

/**
 * Куда ставить мод: сборка (клиент) и/или игровые сервера.
 *
 * Список грузится по выбранному серверу, а не целиком: сборок и инстансов
 * бывает много, а нужны всегда одного пака.
 */
export function useInstallTargets(initialServerId?: string) {
  const auth = useAuth();

  const servers = ref<ServerRow[]>([]);
  const builds = ref<BuildRow[]>([]);
  const gameServers = ref<GameServer[]>([]);
  const serverId = ref(initialServerId ?? "");
  const loading = ref(false);

  /** Выбранные цели: ключи вида `build:<id>` и `gs:<id>`. */
  const picked = ref<Set<string>>(new Set());

  async function loadServers() {
    servers.value = (await auth.requestList<ServerRow>("/api/admin/servers")) ?? [];
    if (!serverId.value && servers.value.length) {
      serverId.value = servers.value[0].id;
    }
  }

  async function loadForServer() {
    if (!serverId.value) return;
    loading.value = true;
    try {
      const [b, g] = await Promise.all([
        auth.request<BuildRow[]>(`/api/admin/builds?server_id=${serverId.value}`),
        auth.requestList<GameServer>(`/api/admin/servers/${serverId.value}/game-servers`,
        ),
      ]);
      builds.value = b ?? [];
      // Прокси модов не держит — предлагать его как цель нечестно.
      gameServers.value = (g ?? []).filter((s) => s.kind !== "proxy");
    } finally {
      loading.value = false;
    }
  }

  function toggle(key: string) {
    const next = new Set(picked.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    picked.value = next;
  }

  function pickAllServers() {
    const next = new Set(picked.value);
    const every = gameServers.value.every((s) => next.has(`gs:${s.id}`));
    for (const s of gameServers.value) {
      if (every) next.delete(`gs:${s.id}`);
      else next.add(`gs:${s.id}`);
    }
    picked.value = next;
  }

  function targets(optionalFor?: (buildId: string) => InstallTarget | null) {
    const list: InstallTarget[] = [];
    for (const key of picked.value) {
      const [kind, id] = key.split(":");
      if (kind === "build") {
        list.push(optionalFor?.(id) ?? { kind: "build", build_id: id });
      } else {
        list.push({ kind: "game_server", id });
      }
    }
    return list;
  }

  watch(serverId, loadForServer);

  return {
    servers,
    builds,
    gameServers,
    serverId,
    picked,
    loading,
    loadServers,
    loadForServer,
    toggle,
    pickAllServers,
    targets,
  };
}
