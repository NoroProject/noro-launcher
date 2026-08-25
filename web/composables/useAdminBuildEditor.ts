import type { BuildFileRow, BuildRow, OptionalMod } from "~/types/api";
import type { ImportProgress, PackImportKind } from "~/types/server-settings";

export function useAdminBuildEditor(
    serverId: Ref<string>,
    buildId: Ref<string | null>,
    onBuildDeleted?: () => void,
) {
    const auth = useAuth();
    const toast = useToast();

    const fileUpload = ref<File | null>(null);
    const filePath = ref("");
    const importFile = ref<File | null>(null);
    const importKind = ref<PackImportKind>("mrpack");
    const busy = ref<string | null>(null);
    const message = ref<string | null>(null);
    const messageError = ref(false);
    const showFileManager = ref(false);
    const importProgress = ref<ImportProgress | null>(null);
    let importTimer: ReturnType<typeof setInterval> | null = null;

    const pathsForm = reactive({ unmanaged: "", userManaged: "" });
    const recommendedForm = reactive({
        memoryMin: 2048,
        memoryMax: 4096,
        jvmFlags: "",
        showConsole: true,
    });

    const buildPayload = useAsyncData(
        async () => {
            if (!buildId.value) return null;
            return auth.request<{ build: BuildRow; file_count: number }>(
                `/api/admin/builds/${buildId.value}`,
            );
        },
        { watch: [buildId] },
    );

    const filesData = useAsyncData(
        async () => {
            if (!buildId.value) return [];
            return auth.request<BuildFileRow[]>(
                `/api/admin/builds/${buildId.value}/files`,
            );
        },
        { default: () => [], watch: [buildId] },
    );

    const optionalData = useAsyncData(
        async () => {
            if (!buildId.value) return [];
            return auth.request<OptionalMod[]>(
                `/api/admin/builds/${buildId.value}/optional-mods`,
            );
        },
        { default: () => [], watch: [buildId] },
    );

    const build = computed(() => buildPayload.data.value?.build);

    watchEffect(() => {
        if (!build.value) return;
        pathsForm.unmanaged = Array.isArray(build.value.unmanaged_paths)
            ? build.value.unmanaged_paths.join("\n")
            : "";
        pathsForm.userManaged = Array.isArray(build.value.user_managed_paths)
            ? build.value.user_managed_paths.join("\n")
            : "";
        recommendedForm.memoryMin = build.value.recommended_memory_min_mb ?? 2048;
        recommendedForm.memoryMax = build.value.recommended_memory_max_mb ?? 4096;
        recommendedForm.jvmFlags = build.value.recommended_jvm_flags ?? "";
        recommendedForm.showConsole =
            build.value.recommended_show_console_on_launch ?? true;
    });

    async function run(name: string, action: () => Promise<void>) {
        busy.value = name;
        message.value = null;
        messageError.value = false;
        try {
            await action();
            message.value = "Done";
        } catch (e) {
            // Без этого упавший запрос выглядит как «кнопка ничего не делает».
            messageError.value = true;
            message.value = e instanceof Error ? e.message : String(e);
        } finally {
            busy.value = null;
        }
    }

    async function publish() {
        if (!buildId.value) return;
        await run("publish", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}/publish`, {
                method: "POST",
            });
            await buildPayload.refresh();
            await filesData.refresh();
        });
    }

    async function rebuild() {
        if (!buildId.value) return;
        await run("rebuild", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}/rebuild`, {
                method: "POST",
            });
            await buildPayload.refresh();
            await filesData.refresh();
        });
    }

    async function rebuildClean() {
        if (!buildId.value) return;
        if (
            !confirm(
                "Discard the Minecraft and loader base and download it again from scratch?\n\n" +
                    "Mods and configs are kept. The file cache is ignored, so this takes minutes — " +
                    "and until it finishes, players cannot fetch this build.",
            )
        )
            return;
        await run("rebuild-clean", async () => {
            await auth.request(
                `/api/admin/builds/${buildId.value}/rebuild-clean`,
                { method: "POST" },
            );
            await buildPayload.refresh();
            await filesData.refresh();
        });
    }

    async function unpublish() {
        if (!buildId.value) return;
        await run("unpublish", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}/unpublish`, {
                method: "POST",
            });
            await buildPayload.refresh();
        });
    }

    async function deleteBuild() {
        if (!buildId.value) return;
        await run("delete", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}`, {
                method: "DELETE",
            });
            if (onBuildDeleted) onBuildDeleted();
        });
    }

    async function uploadBuildFile() {
        if (!buildId.value || !fileUpload.value) return;
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
        if (!buildId.value) return;
        await run(`delete-file-${fileId}`, async () => {
            await auth.request(
                `/api/admin/builds/${buildId.value}/files/${fileId}`,
                { method: "DELETE" },
            );
            await filesData.refresh();
        });
    }

    async function importPack() {
        if (!buildId.value || !importFile.value) return;
        const segment = importKind.value;
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
                                `/api/admin/builds/${buildId.value}/import-progress/${resp.job_id}`,
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

    async function applyPackVersions() {
        if (!buildId.value || !importProgress.value?.recommended_mc_version || !importProgress.value?.recommended_modloader_version) return;
        await run("apply-versions", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}/versions`, {
                method: "PUT",
                body: {
                    mc_version: importProgress.value!.recommended_mc_version,
                    modloader_version: importProgress.value!.recommended_modloader_version,
                },
            });
            await buildPayload.refresh();
            importProgress.value!.recommended_mc_version = null;
            importProgress.value!.recommended_modloader_version = null;
            toast.add({
                title: "Versions Applied",
                description: "You need to rebuild the pack for changes to take effect.",
                icon: "i-lucide-check-circle",
                color: "success",
                duration: 5000,
            });
        });
    }

    async function saveRecommended() {
        if (!buildId.value) return;
        await run("recommended", async () => {
            await auth.request(
                `/api/admin/builds/${buildId.value}/recommended-settings`,
                {
                    method: "PUT",
                    body: {
                        memory_min_mb: recommendedForm.memoryMin,
                        memory_max_mb: recommendedForm.memoryMax,
                        jvm_flags: recommendedForm.jvmFlags,
                        show_console_on_launch: recommendedForm.showConsole,
                    },
                },
            );
            await buildPayload.refresh();
        });
    }

    async function saveOptional() {
        if (!buildId.value) return;
        await run("optional", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}/optional-mods`, {
                method: "PUT",
                body: optionalData.data.value,
            });
            await optionalData.refresh();
        });
    }

    async function toggleAllowSuggestions(allow: boolean) {
        if (!buildId.value) return;
        await run("optional-toggle", async () => {
            await auth.request(`/api/admin/builds/${buildId.value}/allow-suggestions`, {
                method: "PUT",
                body: { allow },
            });
            await buildPayload.refresh();
        });
    }

    function deleteOptional(index: number) {
        if (optionalData.data.value) {
            optionalData.data.value.splice(index, 1);
        }
    }

    onUnmounted(() => {
        if (importTimer) clearInterval(importTimer);
    });

    return {
        buildPayload,
        filesData,
        optionalData,
        build,
        fileUpload,
        filePath,
        importFile,
        importKind,
        busy,
        message,
        messageError,
        showFileManager,
        importProgress,
        pathsForm,
        recommendedForm,
        publish,
        rebuild,
        rebuildClean,
        unpublish,
        deleteBuild,
        uploadBuildFile,
        removeFile,
        importPack,
        applyPackVersions,
        saveRecommended,
        saveOptional,
        deleteOptional,
        toggleAllowSuggestions,
    };
}
