import type {
  CatalogCategory,
  CatalogPage,
  CatalogProviders,
  ModHit,
} from "~/types/catalog";

export const PAGE_SIZE = 20;

/** Контекст сборки: подставляется в фильтры совместимости при открытии. */
export interface CatalogContext {
  mc?: string | null;
  loader?: string | null;
}

export function useModCatalog(context: () => CatalogContext) {
  const auth = useAuth();

  const filters = reactive({
    q: "",
    /** `modrinth` | `curseforge` | `all`. */
    provider: "modrinth",
    projectType: "mod",
    mc: context().mc ?? "",
    loader: context().loader ?? "",
    categories: [] as string[],
    side: "" as "" | "client" | "server",
    sort: "relevance",
  });

  const offset = ref(0);
  const hits = ref<ModHit[]>([]);
  const total = ref(0);
  const failed = ref<string[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const categories = ref<CatalogCategory[]>([]);
  const providers = ref<CatalogProviders>({ modrinth: true, curseforge: false });

  /** Категории у провайдеров свои: у Modrinth slug, у CurseForge числовой id.
   *  В смешанном режиме берём список Modrinth — фильтр применится к нему. */
  const facetProvider = computed(() =>
    filters.provider === "curseforge" ? "curseforge" : "modrinth",
  );

  function queryString() {
    const params = new URLSearchParams({
      q: filters.q,
      provider: filters.provider,
      project_type: filters.projectType,
      sort: filters.sort,
      offset: String(offset.value),
      limit: String(PAGE_SIZE),
    });
    if (filters.mc) params.set("mc", filters.mc);
    if (filters.loader) params.set("loader", filters.loader);
    if (filters.side) params.set("side", filters.side);
    if (filters.categories.length) {
      params.set("categories", filters.categories.join(","));
    }
    return params.toString();
  }

  async function search() {
    loading.value = true;
    error.value = null;
    try {
      const page = await auth.request<CatalogPage>(
        `/api/admin/catalog/search?${queryString()}`,
      );
      hits.value = page?.hits ?? [];
      total.value = page?.total ?? 0;
      failed.value = page?.failed ?? [];
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      hits.value = [];
      total.value = 0;
    } finally {
      loading.value = false;
    }
  }

  async function loadCategories() {
    const url = `/api/admin/catalog/categories?provider=${facetProvider.value}&project_type=${filters.projectType}`;
    categories.value = (await auth.request<CatalogCategory[]>(url)) ?? [];
  }

  async function loadProviders() {
    const known = await auth.request<CatalogProviders>(
      "/api/admin/catalog/providers",
    );
    if (known) providers.value = known;
    // Без ключа CurseForge смешанный режим наполовину мёртв — молча падаем
    // обратно на Modrinth, а не показываем половину выдачи с ошибкой.
    if (!providers.value.curseforge && filters.provider !== "modrinth") {
      filters.provider = "modrinth";
    }
  }

  function toggleCategory(name: string) {
    const at = filters.categories.indexOf(name);
    if (at >= 0) filters.categories.splice(at, 1);
    else filters.categories.push(name);
  }

  function resetFilters() {
    filters.categories = [];
    filters.side = "";
    filters.mc = context().mc ?? "";
    filters.loader = context().loader ?? "";
  }

  function goToPage(next: number) {
    offset.value = Math.max(0, next) * PAGE_SIZE;
  }

  const page = computed(() => Math.floor(offset.value / PAGE_SIZE));
  const pageCount = computed(() => Math.ceil(total.value / PAGE_SIZE));

  let debounce: ReturnType<typeof setTimeout> | null = null;
  watch(
    () => filters.q,
    () => {
      if (debounce) clearTimeout(debounce);
      // Полсекунды: за меньшее Modrinth успевает получить запрос на каждую
      // букву и начинает отвечать 429.
      debounce = setTimeout(() => {
        offset.value = 0;
        search();
      }, 400);
    },
  );

  // Остальные фильтры меняются кликом, ждать нечего.
  watch(
    () => [
      filters.provider,
      filters.projectType,
      filters.mc,
      filters.loader,
      filters.side,
      filters.sort,
      filters.categories.join(","),
    ],
    () => {
      offset.value = 0;
      search();
    },
  );
  watch([facetProvider, () => filters.projectType], loadCategories);
  watch(offset, search);

  return {
    filters,
    hits,
    total,
    failed,
    loading,
    error,
    categories,
    providers,
    page,
    pageCount,
    search,
    loadCategories,
    loadProviders,
    toggleCategory,
    resetFilters,
    goToPage,
  };
}
