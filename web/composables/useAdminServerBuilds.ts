import type { ComputedRef, Ref } from "vue";
import type { BuildRow } from "~/types/api";
import type { BuildCreateForm } from "~/types/server-settings";

type AuthApi = ReturnType<typeof useAuth>;

function createBuildForm(serverId: string): BuildCreateForm {
  return {
    server_id: serverId,
    version: "",
    modloader: "fabric",
    modloader_version: "",
    mc_version: "1.21.1",
    copy_from: "",
  };
}

export function useAdminServerBuilds(
  auth: AuthApi,
  id: ComputedRef<string>,
  loader: Ref<Record<string, string[]>>,
  loadLoader: (kind: string, mc: string) => Promise<void>,
) {
  const notify = useNotify();

  const buildsData = useAsyncData(
    `admin-builds-${id.value}`,
    () => auth.request<BuildRow[]>(`/api/admin/builds?server_id=${id.value}`),
    { default: () => [] },
  );
  const buildForm = reactive(createBuildForm(id.value));
  const creatingBuild = ref(false);
  const showCreateBuild = ref(false);
  const loaderOptions = computed(() => loader.value[buildForm.modloader] || []);

  watch(
    () => [buildForm.modloader, buildForm.mc_version],
    ([kind, mc]) => loadLoader(kind, mc).catch(() => {}),
    { immediate: true },
  );

  async function createBuild() {
    creatingBuild.value = true;
    try {
      if (buildForm.copy_from) {
        // Копия наследует загрузчик, версии, пути и весь список файлов —
        // мастер переиспользует те же объекты FileStore, ничего не перезаливая.
        await auth.request(
          `/api/admin/builds/${buildForm.copy_from}/duplicate`,
          { method: "POST", body: { version: buildForm.version } },
        );
      } else {
        await auth.request("/api/admin/builds", {
          method: "POST",
          body: {
            ...buildForm,
            modloader_version: buildForm.modloader_version || null,
          },
        });
      }
      buildForm.version = "";
      buildForm.copy_from = "";
      await buildsData.refresh();
      showCreateBuild.value = false;
      notify.ok()
    } catch (e) {
      notify.fail(e)
    } finally {
      creatingBuild.value = false;
    }
  }

  return {
    buildsData,
    buildForm,
    loaderOptions,
    creatingBuild,
    showCreateBuild,
    createBuild,
  };
}
