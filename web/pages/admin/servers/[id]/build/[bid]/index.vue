<script setup lang="ts">
import type { BuildFileRow, BuildRow, OptionalMod } from "~/types/api";
import type { ImportProgress, PackImportKind } from "~/types/server-settings";

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
        auth.request<BuildFileRow[]>(
            `/api/admin/builds/${buildId.value}/files`,
        ),
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
const showFileManager = ref(false);
const pathsForm = reactive({ unmanaged: "", userManaged: "" });
const recommendedForm = reactive({
    memoryMin: 2048,
    memoryMax: 4096,
    jvmFlags: "",
    showConsole: true,
});
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
        await filesData.refresh();
    });
}

async function rebuild() {
    await run("rebuild", async () => {
        await auth.request(`/api/admin/builds/${buildId.value}/rebuild`, {
            method: "POST",
        });
        await buildPayload.refresh();
        await filesData.refresh();
    });
}

async function rebuildClean() {
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
        await navigateTo(`/admin/servers/${serverId.value}`);
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
        await auth.request(
            `/api/admin/builds/${buildId.value}/files/${fileId}`,
            { method: "DELETE" },
        );
        await filesData.refresh();
    });
}

const importProgress = ref<ImportProgress | null>(null);
let importTimer: ReturnType<typeof setInterval> | null = null;

async function importPack() {
    if (!importFile.value) return;
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

const toast = useToast();

async function applyPackVersions() {
    if (!importProgress.value?.recommended_mc_version || !importProgress.value?.recommended_modloader_version) return;
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
    await run("optional", async () => {
        await auth.request(`/api/admin/builds/${buildId.value}/optional-mods`, {
            method: "PUT",
            body: optionalData.data.value,
        });
        await optionalData.refresh();
    });
}

function deleteOptional(index: number) {
    if (optionalData.data.value) {
        optionalData.data.value.splice(index, 1);
    }
}
async function toggleAllowSuggestions(allow: boolean) {
    await run("optional-toggle", async () => {
        await auth.request(`/api/admin/builds/${buildId.value}/allow-suggestions`, {
            method: "PUT",
            body: { allow },
        });
        await buildPayload.refresh();
    });
}
</script>

<template>
    <NoroShell
        :title="`BUILD ${build?.version || buildId}`"
        :subtitle="build?.published ? 'published' : 'draft'"
    >
        <template #actions>
            <AtomButton
              variant="dark"
              icon="i-lucide-arrow-left"
              :to="`/admin/servers/${serverId}`"
            >
              Back
            </AtomButton>
        </template>

        <UAlert
            v-if="message"
            class="mb-5"
            color="success"
            variant="subtle"
            icon="i-lucide-check"
            :description="message"
        />

        <div v-if="importProgress && !importProgress.done" class="mb-5 noro-panel p-5 bg-noro-blue/10 border-noro-blue/30">
            <h3 class="text-sm font-bold text-noro-blue mb-2 flex items-center gap-2">
                <UIcon name="i-lucide-loader-2" class="animate-spin" />
                Importing Modpack...
            </h3>
            <div class="mb-1 text-xs text-noro-muted flex justify-between">
                <span class="truncate pr-4">{{ importProgress.current_file }}</span>
                <span class="shrink-0">{{ importProgress.current }} / {{ importProgress.total }}</span>
            </div>
            <UProgress
                :value="importProgress.current"
                :max="importProgress.total || 1"
                color="info"
                size="sm"
            />
            <div v-if="importProgress.warnings.length > 0" class="mt-4 pt-4 border-t border-noro-border/50">
                <p class="text-xs font-bold text-noro-warning mb-2">Warnings:</p>
                <ul class="text-xs text-noro-muted list-disc list-inside space-y-1">
                    <li v-for="(warn, i) in importProgress.warnings" :key="i" class="text-noro-warning/80">
                        {{ warn }}
                    </li>
                </ul>
            </div>
        </div>
        <div v-else-if="importProgress && importProgress.warnings.length > 0" class="mb-5 noro-panel p-5 bg-noro-warning/10 border-noro-warning/30">
            <h3 class="text-sm font-bold text-noro-warning mb-2 flex items-center gap-2">
                <UIcon name="i-lucide-alert-triangle" />
                Import Completed with Warnings
            </h3>
            <ul class="text-xs text-noro-muted list-disc list-inside space-y-1">
                <li v-for="(warn, i) in importProgress.warnings" :key="i" class="text-noro-warning/80">
                    {{ warn }}
                </li>
            </ul>
            <AtomButton variant="ghost" size="sm" class="mt-3" @click="importProgress = null">Dismiss</AtomButton>
        </div>

        <!-- Version Suggestions -->
        <div 
            v-if="importProgress && (importProgress.recommended_mc_version || importProgress.recommended_modloader_version) && 
                 (importProgress.recommended_mc_version !== build?.mc_version || importProgress.recommended_modloader_version !== build?.modloader_version)" 
            class="mb-5 noro-panel p-5 bg-noro-primary/10 border-noro-primary/30 flex items-center justify-between"
        >
            <div>
                <h3 class="text-sm font-bold text-noro-primary mb-1 flex items-center gap-2">
                    <UIcon name="i-lucide-info" />
                    Recommended Versions
                </h3>
                <p class="text-xs text-noro-muted">
                    The imported modpack uses 
                    <strong v-if="importProgress.recommended_mc_version">Minecraft {{ importProgress.recommended_mc_version }}</strong>
                    <span v-if="importProgress.recommended_mc_version && importProgress.recommended_modloader_version"> and </span>
                    <strong v-if="importProgress.recommended_modloader_version">Modloader {{ importProgress.recommended_modloader_version }}</strong>.
                    Would you like to apply these settings to your build?
                </p>
            </div>
            <AtomButton variant="primary" size="sm" @click="applyPackVersions" :loading="busy === 'apply-versions'">
                Apply Versions
            </AtomButton>
        </div>

        <div class="grid gap-5 xl:grid-cols-[1fr_390px] items-start">
            <div class="grid gap-4 content-start">
                <BuildPublishPanel
                    :build="build"
                    :busy="busy"
                    :build-pending="buildPayload.pending.value"
                    @publish="publish"
                    @rebuild="rebuild"
                    @rebuild-clean="rebuildClean"
                    @unpublish="unpublish"
                    @delete="deleteBuild"
                />
                <BuildFilesSummary
                  :files="filesData.data.value"
                  :build-id="buildId"
                  @open-manager="showFileManager = true"
                />
                <BuildFileManagerModal v-model="showFileManager" :build-id="buildId" @changed="filesData.refresh()" />
                <BuildOptionalModsPanel
                    :optional-mods="optionalData.data.value"
                    :allow-suggestions="build?.allow_optional_mod_suggestions ?? true"
                    :busy="busy"
                    @save="saveOptional"
                    @delete="deleteOptional"
                    @update:allow-suggestions="toggleAllowSuggestions"
                />
                <BuildRecommendedSettingsPanel
                    :form="recommendedForm"
                    :busy="busy"
                    @save="saveRecommended"
                />
                <BuildPathsPanel
                    :paths-form="pathsForm"
                    @browse="showFileManager = true"
                />
            </div>

            <aside class="grid gap-5 content-start">
                <BuildImportPanel
                    v-model:import-file="importFile"
                    v-model:import-kind="importKind"
                    :busy="busy"
                    @submit-import="importPack"
                />
                <BuildManualFilePanel
                    v-model:file-upload="fileUpload"
                    v-model:file-path="filePath"
                    :busy="busy"
                    @upload="uploadBuildFile"
                />
                <BuildModCatalogPanel
                    :to="`/admin/servers/${serverId}/build/${buildId}/mods`"
                />
            </aside>
        </div>
    </NoroShell>
</template>
