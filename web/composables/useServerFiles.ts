import type { ServerOutcome, ServerEntry, ServerListing } from "~/types/wrapper";

/**
 * Файлы игрового сервера: навигация, чтение, запись, удаление.
 *
 * Путей мы не проверяем — этим занимается враппер, у которого есть настоящая
 * файловая система. Повторять проверку здесь значит однажды разойтись с ней.
 */
export function useServerFiles(gameServerId: string) {
  const auth = useAuth();
  const notify = useNotify();
  const base = `/api/admin/game-servers/${gameServerId}/fs`;

  const path = ref(".");
  const entries = ref<ServerEntry[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  /** Хлебные крошки: «.» это корень, дальше сегменты пути. */
  const crumbs = computed(() => {
    if (path.value === ".") return [] as { label: string; path: string }[];
    const parts = path.value.split("/").filter(Boolean);
    return parts.map((label, i) => ({ label, path: parts.slice(0, i + 1).join("/") }));
  });

  async function open(next: string) {
    loading.value = true;
    error.value = null;
    try {
      const listing = await auth.request<ServerListing>(
        `${base}?path=${encodeURIComponent(next)}`,
      );
      entries.value = listing?.entries ?? [];
      path.value = listing?.path ?? next;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      entries.value = [];
    } finally {
      loading.value = false;
    }
  }

  function up() {
    const parts = path.value.split("/").filter((p) => p && p !== ".");
    parts.pop();
    return open(parts.length ? parts.join("/") : ".");
  }

  async function read(file: string) {
    const data = await auth.request<{ path: string; content: string }>(
      `${base}/file?path=${encodeURIComponent(file)}`,
    );
    return data?.content ?? "";
  }

  async function write(file: string, content: string) {
    await auth.request(`${base}/file`, {
      method: "PUT",
      body: { path: file, content },
    });
    notify.ok("Saved", file);
  }

  /** Тот же файл на несколько серверов сборки. */
  async function apply(file: string, content: string, targets: string[]) {
    const report = await auth.request<ServerOutcome[]>(`${base}/apply`, {
      method: "POST",
      body: { path: file, content, targets },
    });
    const failed = (report ?? []).filter((r) => !r.ok);
    if (failed.length) {
      notify.fail(
        new Error(failed.map((r) => r.error).join("; ")),
        `Applied to ${(report?.length ?? 0) - failed.length} of ${report?.length ?? 0}`,
      );
    } else {
      notify.ok("Applied", `${report?.length ?? 0} server(s)`);
    }
    return report ?? [];
  }

  async function remove(target: string) {
    await auth.request(`${base}?path=${encodeURIComponent(target)}`, { method: "DELETE" });
    notify.ok("Deleted", target);
    await open(path.value);
  }

  async function mkdir(name: string) {
    const next = path.value === "." ? name : `${path.value}/${name}`;
    await auth.request(`${base}/mkdir`, { method: "POST", body: { path: next } });
    await open(path.value);
  }

  return { path, entries, crumbs, loading, error, open, up, read, write, apply, remove, mkdir };
}
