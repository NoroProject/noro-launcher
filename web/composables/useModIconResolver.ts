import type { BuildFileRow } from "~/types/api";

const iconCache = new Map<string, string>();

/**
 * Extracts a human-readable mod name from file path or jar filename.
 * e.g. "mods/FarmersDelight-1.21.1-1.3.2.jar" -> "Farmers Delight"
 */
export function cleanModTitle(path: string): string {
    const filename = path.replace(/^mods\//i, "").replace(/\.jar$/i, "");

    const lower = filename.toLowerCase();
    if (lower.includes("xaerominimap")) return "Xaero's Minimap";
    if (lower.includes("xaeroworldmap")) return "Xaero's World Map";
    if (lower.includes("xaerotrainmap")) return "Xaero's Train Map";
    if (lower.includes("ftbteams")) return "FTB Teams";
    if (lower.includes("kotlinforforge")) return "Kotlin For Forge";
    if (lower.includes("tinymobfarm")) return "Tiny Mob Farm";
    if (lower.includes("nomobfarm")) return "No Mob Farm";
    if (lower.includes("trophymanager")) return "Trophy Manager";
    if (lower.includes("powergrid")) return "Power Grid";
    if (lower.includes("tfmgfixfuel")) return "TFMG Fix Fuel";
    if (lower.includes("tfmg")) return "The Factory Must Grow";
    if (lower.includes("sablemassview")) return "Sable Mass View";
    if (lower.includes("smoothmovement")) return "Smooth Movement";
    if (lower.includes("plasticfix")) return "Plastic Fix";
    if (lower.includes("yamlconfig")) return "Yaml Config";
    if (lower.includes("uilib")) return "UiLib";
    if (lower.includes("aeronautics")) return "Create Aeronautics";
    if (lower.includes("copycat")) return "Copycat";
    if (lower.includes("sodium")) return "Sodium";
    if (lower.includes("spark")) return "spark";

    const cleaned = filename
        .replace(/[-_](neoforge|forge|fabric|quilt).*/i, "")
        .replace(/[-_]\d+\.\d+(\.\d+)?.*$/i, "")
        .replace(/([a-z])([A-Z])/g, "$1 $2")
        .replace(/[-_]+/g, " ")
        .trim();

    return cleaned || filename;
}

export function useModIconResolver() {
    const icons = ref<Record<string, string>>({});
    const auth = useAuth();

    async function resolveModIcons(files: BuildFileRow[], buildId?: string | null) {
        const modsToResolve = files.filter((f) => {
            const isJar = f.path.toLowerCase().endsWith(".jar");
            const inMods = f.path.toLowerCase().startsWith("mods/");
            return (isJar || inMods || f.kind === "mod") && f.sha1;
        });

        if (!modsToResolve.length) return;

        // 1. Populate from cache
        for (const file of modsToResolve) {
            if (iconCache.has(file.sha1)) {
                icons.value[file.sha1] = iconCache.get(file.sha1)!;
            }
        }

        const missingHashes = modsToResolve
            .map((f) => f.sha1)
            .filter((h) => h && !iconCache.has(h));

        if (!missingHashes.length && !buildId) return;

        // 2. Modrinth SHA1 Exact Lookup in batches of 100
        for (let i = 0; i < missingHashes.length; i += 100) {
            const hashBatch = missingHashes.slice(i, i + 100);
            try {
                const res = await $fetch<Record<string, { project_id: string }>>(
                    "https://api.modrinth.com/v2/version_files",
                    {
                        method: "POST",
                        body: { hashes: hashBatch, algorithm: "sha1" },
                    },
                );

                const projectIds = Array.from(
                    new Set(Object.values(res).map((v) => v.project_id)),
                ).filter(Boolean);

                if (projectIds.length) {
                    const projects = await $fetch<Array<{ id: string; icon_url?: string }>>(
                        `https://api.modrinth.com/v2/projects?ids=${JSON.stringify(projectIds)}`,
                    );

                    const projMap = new Map(projects.map((p) => [p.id, p.icon_url]));
                    for (const [hash, info] of Object.entries(res)) {
                        const icon = projMap.get(info.project_id);
                        if (icon) {
                            iconCache.set(hash, icon);
                            icons.value[hash] = icon;
                        }
                    }
                }
            } catch {
                // Ignore API network errors
            }
        }

        // 3. Fallback Search for ALL remaining unresolved mods in batches of 10
        let unresolved = modsToResolve.filter((f) => !icons.value[f.sha1]);
        for (let i = 0; i < unresolved.length; i += 10) {
            const searchBatch = unresolved.slice(i, i + 10);
            await Promise.allSettled(
                searchBatch.map(async (file) => {
                    const title = cleanModTitle(file.path);
                    if (!title || title.length < 2) return;

                    try {
                        const searchRes = await $fetch<{
                            hits?: Array<{ icon_url?: string }>;
                        }>(`https://api.modrinth.com/v2/search?query=${encodeURIComponent(title)}&limit=1`);

                        const firstHit = searchRes.hits?.[0];
                        if (firstHit?.icon_url) {
                            iconCache.set(file.sha1, firstHit.icon_url);
                            icons.value[file.sha1] = firstHit.icon_url;
                        }
                    } catch {
                        // Ignore search failure
                    }
                }),
            );
        }

        // 4. Backend Inner Jar Icon Extraction for ALL remaining unresolved mods in batches of 10
        if (buildId) {
            unresolved = modsToResolve.filter((f) => !icons.value[f.sha1]);
            for (let i = 0; i < unresolved.length; i += 10) {
                const innerBatch = unresolved.slice(i, i + 10);
                await Promise.allSettled(
                    innerBatch.map(async (file) => {
                        try {
                            const res = await auth.request<{ icon_url?: string }>(
                                `/api/admin/builds/${buildId}/files/icon?path=${encodeURIComponent(file.path)}`,
                            );
                            if (res?.icon_url) {
                                iconCache.set(file.sha1, res.icon_url);
                                icons.value[file.sha1] = res.icon_url;
                            }
                        } catch {
                            // Ignore missing inner icon
                        }
                    }),
                );
            }
        }
    }

    return {
        icons,
        resolveModIcons,
    };
}
