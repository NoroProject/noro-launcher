import type { BuildFileRow, BuildRow, OptionalMod } from "~/types/api";
import type { ImportProgress, PackImportKind } from "~/types/server-settings";

export async function useBuildEditor() {
  const route = useRoute();
  const auth = useAuth();
  await auth.loadMe();

  const serverId = computed(() => String(route.params.id));
  const buildId = computed(() => String(route.params.bid));
  const buildPayload = await useAsyncData(`admin-build-${buildId.value}`, () =>
    auth.request<{ build: BuildRow; file_count: number }>(
      `/api/admin/builds/${buildId.value}`,
    ),
  );
  const filesData = await useAsyncData(
    `admin-build-files-${buildId.value}`,
    () =>
      auth.request<BuildFileRow[]>(`/api/admin/builds/${buildId.value}/files`),
    { default: () => [] },
  );
  const optionalData = await useAsyncData(
    `admin-build-optional-${buildId.value}`,
    () =>
      auth.request<OptionalMod[]>(
        `/api/admin/builds/${buildId.value}/optional-mods`,
      ),
    { default: () => [] },
  );

  const fileUpload = ref<File | null>(null);
  const filePath = ref("");
  const importFile = ref<File | null>(null);
  const importKind = ref<PackImportKind>("mrpack");
  const busy = ref<string | null>(null);
  const message = ref<string | null>(null);
  const pathsForm = reactive({ unmanaged: "", userManaged: "" });
  const build = computed(() => buildPayload.data.value?.build);

  watchEffect(() => {
    if (!build.value) return;
    pathsForm.unmanaged = Array.isArray(build.value.unmanaged_paths)
      ? build.value.unmanaged_paths.join("\n")
      : "";
    pathsForm.userManaged = Array.isArray(build.value.user_managed_paths)
      ? build.value.user_managed_paths.join("\n")
      : "";
  });

  function lines(value: string) {
    return value
      .split("\n")
      .map((s) => s.trim())
      .filter(Boolean);
  }

  async function run(name: string, action: () => Promise<void>) {
    busy.value = name;
    message.value = null;
    try {
      await action();
      message.value = "Done";
    } finally {
      busy.value = null;
    }
  }
  async function publish() {
    await run("publish", async () => {
      await auth.request(`/api/admin/builds/${buildId.value}/publish`, {
        method: "POST",
      });
      await buildPayload.refresh();
    });
  }

  async function unpublish() {
    await run("unpublish", async () => {
      await auth.request(`/api/admin/builds/${buildId.value}/unpublish`, {
        method: "POST",
      });
      await buildPayload.refresh();
    });
  }

  async function deleteBuild() {
    await run("delete", async () => {
      await auth.request(`/api/admin/builds/${buildId.value}`, {
        method: "DELETE",
      });
      await navigateTo(`/admin/clients/${serverId.value}`);
    });
  }

  async function uploadBuildFile() {
    if (!fileUpload.value) return;
    await run("file", async () => {
      const extra = filePath.value ? { path: filePath.value } : undefined;
      await auth.upload(
        `/api/admin/builds/${buildId.value}/files`,
        "file",
        fileUpload.value!,
        extra,
      );
      fileUpload.value = null;
      filePath.value = "";
      await filesData.refresh();
    });
  }

  async function removeFile(fileId: string) {
    await run(`delete-file-${fileId}`, async () => {
      await auth.request(`/api/admin/builds/${buildId.value}/files/${fileId}`, {
        method: "DELETE",
      });
      await filesData.refresh();
    });
  }

  const importProgress = ref<ImportProgress | null>(null);
  let importTimer: ReturnType<typeof setInterval> | null = null;

  async function importPack() {
    if (!importFile.value) return;
    // Имя вкладки совпадает с сегментом маршрута импорта.
    const segment = importKind.value;
    
    // Clear previous state
    importProgress.value = {
      total: 100,
      current: 0,
      current_file: "Starting...",
      done: false,
      warnings: [],
    };
    
    await run("import", async () => {
      const resp = await auth.upload<{ job_id: string }>(
        `/api/admin/builds/${buildId.value}/import/${segment}`,
        "file",
        importFile.value!,
      );
      importFile.value = null;

      if (resp && resp.job_id) {
        return new Promise<void>((resolve, reject) => {
          importTimer = setInterval(async () => {
            try {
              const prog = await auth.request<ImportProgress>(
                `/api/admin/builds/${buildId.value}/import_progress/${resp.job_id}`
              );
              importProgress.value = prog;
              if (prog.done) {
                if (importTimer) clearInterval(importTimer);
                if (prog.error) {
                  message.value = `Error: ${prog.error}`;
                  reject(new Error(prog.error));
                } else {
                  await filesData.refresh();
                  resolve();
                }
              }
            } catch (err) {
              if (importTimer) clearInterval(importTimer);
              reject(err);
            }
          }, 500);
        });
      } else {
        await filesData.refresh();
      }
    });
  }

  async function savePaths() {
    await run("paths", async () => {
      await auth.request(`/api/admin/builds/${buildId.value}/paths`, {
        method: "PUT",
        body: {
          unmanaged_paths: lines(pathsForm.unmanaged),
          user_managed_paths: lines(pathsForm.userManaged),
        },
      });
      await buildPayload.refresh();
    });
  }

  async function saveOptional() {
    await run("optional", async () => {
      await auth.request(`/api/admin/builds/${buildId.value}/optional-mods`, {
        method: "PUT",
        body: optionalData.data.value,
      });
      await optionalData.refresh();
    });
  }

  function addOptional() {
    optionalData.data.value = [
      ...(optionalData.data.value || []),
      {
        name: "",
        description: "",
        category: "Gameplay",
        files: [],
        enabled_by_default: false,
        visible: true,
        limited: false,
        dependencies: [],
        conflicts: [],
        triggers: [],
      },
    ];
  }

  function deleteOptional(index: number) {
    if (optionalData.data.value) {
      optionalData.data.value.splice(index, 1);
    }
  }

  return {
    serverId,
    buildId,
    build,
    buildPending: buildPayload.pending,
    files: filesData.data,
    optionalMods: optionalData.data,
    fileUpload,
    filePath,
    importFile,
    importKind,
    importProgress,
    busy,
    message,
    pathsForm,
    publish,
    unpublish,
    deleteBuild,
    uploadBuildFile,
    removeFile,
    importPack,
    savePaths,
    saveOptional,
    addOptional,
    deleteOptional,
  };
}
