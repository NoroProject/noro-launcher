import type { ModHit, ModProject, ModVersion } from "~/types/catalog";

/**
 * Карточка проекта и его версии.
 *
 * Версии просим отдельно от проекта: их список зависит от того, под какую
 * сборку смотрим, и переключение «только совместимые» не должно перезагружать
 * описание с галереей.
 */
export function useModProject() {
  const auth = useAuth();

  const hit = ref<ModHit | null>(null);
  const project = ref<ModProject | null>(null);
  const versions = ref<ModVersion[]>([]);
  const loading = ref(false);
  const loadingVersions = ref(false);
  const error = ref<string | null>(null);
  /** Сужать список версий до версии игры и загрузчика сборки. */
  const compatibleOnly = ref(true);
  const context = reactive({ mc: "", loader: "" });

  async function loadVersions() {
    if (!hit.value) return;
    loadingVersions.value = true;
    try {
      const params = new URLSearchParams();
      if (compatibleOnly.value) {
        if (context.mc) params.set("mc", context.mc);
        if (context.loader) params.set("loader", context.loader);
      }
      const suffix = params.toString() ? `?${params}` : "";
      versions.value =
        (await auth.request<ModVersion[]>(
          `/api/admin/catalog/${hit.value.provider}/project/${hit.value.project_id}/versions${suffix}`,
        )) ?? [];
    } finally {
      loadingVersions.value = false;
    }
  }

  async function open(next: ModHit, mc: string, loader: string) {
    hit.value = next;
    context.mc = mc;
    context.loader = loader;
    project.value = null;
    versions.value = [];
    error.value = null;
    loading.value = true;
    try {
      project.value = await auth.request<ModProject>(
        `/api/admin/catalog/${next.provider}/project/${next.project_id}`,
      );
      await loadVersions();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  function close() {
    hit.value = null;
    project.value = null;
    versions.value = [];
  }

  /**
   * Самая свежая совместимая версия — для установки в один клик из карточки.
   * Каталоги отдают версии от новых к старым, поэтому берём первую скачиваемую.
   */
  async function fetchLatest(next: ModHit, mc: string, loader: string) {
    const params = new URLSearchParams();
    if (mc) params.set("mc", mc);
    if (loader) params.set("loader", loader);
    const suffix = params.toString() ? `?${params}` : "";
    const list =
      (await auth.request<ModVersion[]>(
        `/api/admin/catalog/${next.provider}/project/${next.project_id}/versions${suffix}`,
      )) ?? [];
    return list.find((v) => v.downloadable) ?? null;
  }

  watch(compatibleOnly, loadVersions);

  return {
    hit,
    project,
    versions,
    loading,
    loadingVersions,
    error,
    compatibleOnly,
    open,
    close,
    fetchLatest,
  };
}
