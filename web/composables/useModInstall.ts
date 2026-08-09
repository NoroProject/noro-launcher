import type { InstallResult, InstallTarget, ModSource } from "~/types/catalog";

/**
 * Установка мода в выбранные цели.
 *
 * Мастер отвечает построчно по каждой цели: часть могла не принять файл, и
 * общий «Failed» соврал бы про остальные.
 */
export function useModInstall() {
  const auth = useAuth();
  const notify = useNotify();

  const installing = ref(false);
  const results = ref<InstallResult[]>([]);

  async function install(source: ModSource, targets: InstallTarget[]) {
    if (!targets.length) {
      notify.fail(new Error("Pick at least one target"), "Nothing selected");
      return false;
    }
    installing.value = true;
    results.value = [];
    try {
      const report =
        (await auth.request<InstallResult[]>("/api/admin/mods/install", {
          method: "POST",
          body: { source, targets },
        })) ?? [];
      results.value = report;

      const failed = report.filter((r) => !r.ok);
      if (failed.length) {
        notify.fail(
          new Error(failed.map((r) => r.error).join("; ")),
          `Installed to ${report.length - failed.length} of ${report.length}`,
        );
      } else {
        notify.ok("Installed", `${report.length} target(s)`);
      }
      return failed.length === 0;
    } catch (e) {
      notify.fail(e, "Install failed");
      return false;
    } finally {
      installing.value = false;
    }
  }

  return { installing, results, install };
}
