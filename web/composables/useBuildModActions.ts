import type { BuildRow } from "~/types/api";
import type { ComputedRef } from "vue";

type Runner = (name: string, action: () => Promise<void>) => Promise<void>;

export function useBuildModActions(
  auth: ReturnType<typeof useAuth>,
  buildId: ComputedRef<string>,
  _build: ComputedRef<BuildRow | undefined>,
  refreshFiles: () => Promise<void>,
  run: Runner,
) {
  async function addDirectUrl(url: string) {
    if (!url) return;
    await run("url", async () => {
      await auth.request(`/api/admin/builds/${buildId.value}/mods/add-url`, {
        method: "POST",
        body: { url },
      });
      await refreshFiles();
    });
  }

  async function addCurseForgeFile(projectId: string, fileId: string) {
    if (!projectId || !fileId) return;
    await run("curseforge-add", async () => {
      await auth.request(
        `/api/admin/builds/${buildId.value}/mods/add-curseforge`,
        {
          method: "POST",
          body: {
            project_id: Number(projectId),
            file_id: Number(fileId),
          },
        },
      );
      await refreshFiles();
    });
  }

  return { addDirectUrl, addCurseForgeFile };
}
