import type { ServerRow } from "~/types/api";
import type {
  ServerAssetKind,
  ServerEditForm,
  ServerSettingsTab,
} from "~/types/server-settings";

export const adminServerModloaders = [
  "vanilla",
  "fabric",
  "neoforge",
  "forge",
  "quilt",
];

function createServerForm(): ServerEditForm {
  return {
    name: "",
    description: "",
    modloader: "fabric",
    mc_version: "1.21.1",
    active: true,
    limited: false,
    sort_order: 0,
  };
}

function hydrateForm(form: ServerEditForm, server: ServerRow) {
  Object.assign(form, {
    name: server.name,
    description: server.description,
    modloader: server.modloader,
    mc_version: server.mc_version,
    active: server.active,
    limited: server.limited,
    sort_order: server.sort_order,
  });
}

export function useAdminServerBase() {
  const route = useRoute();
  const auth = useAuth();
  const notify = useNotify()

  const id = computed(() => String(route.params.id));
  const activeTab = ref<ServerSettingsTab>(
    (route.query.tab as ServerSettingsTab) || "server",
  );
  watch(
    () => route.query.tab,
    (t) => {
      if (t && ["server", "mods", "build", "instances", "client"].includes(t as string)) {
        activeTab.value = t as ServerSettingsTab;
      }
    },
  );
  watch(activeTab, (t) => {
    if (route.query.tab !== t) {
      navigateTo({ query: { ...route.query, tab: t } }, { replace: true });
    }
  });
  const versions = useVersionOptions();
  
  const serversData = useAsyncData(
    "admin-server-list-for-edit",
    () => auth.request<ServerRow[]>("/api/admin/servers"),
    { default: () => [] },
  );
  const servers = serversData.data;
  const refresh = serversData.refresh;

  const server = computed(() =>
    servers.value.find((item) => item.id === id.value),
  );

  const form = reactive(createServerForm());
  watchEffect(() => {
    if (server.value) hydrateForm(form, server.value);
  });

  const saving = ref(false);
  const deleting = ref(false);
  onMounted(versions.loadMinecraft);

  async function save() {
    saving.value = true;
    try {
      await auth.request(`/api/admin/servers/${id.value}`, {
        method: "PUT",
        body: form,
      });
      await refresh();
      notify.ok()
    } catch (e) {
      notify.fail(e)
    } finally {
      saving.value = false;
    }
  }

  async function remove() {
    deleting.value = true;
    try {
      await auth.request(`/api/admin/servers/${id.value}`, { method: "DELETE" });
      await navigateTo("/admin/servers");
      notify.ok()
    } catch (e) {
      notify.fail(e)
    } finally {
      deleting.value = false;
    }
  }

  function applyAsset(kind: ServerAssetKind, url: string) {
    if (!server.value) return;
    if (kind === "icon") server.value.icon_url = url;
    else server.value.background_url = url;
  }

    return {
    auth,
    id,
    server,
    form,
    activeTab,
    modloaders: adminServerModloaders,
    save,
    remove,
    saving,
    deleting,
    applyAsset,
    serversData,
    ...versions,
  };
}
