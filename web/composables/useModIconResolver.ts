import type { BuildFileRow } from "~/types/api";

const iconCache = new Map<string, string>();

/**
 * Extracts a human-readable mod name from file path or jar filename.
 * e.g. "mods/FarmersDelight-1.21.1-1.3.2.jar" -> "Farmers Delight"
 */
export function cleanModTitle(path: string): string {
    const filename = path.replace(/^mods\//i, "").replace(/\.jar$/i, "");
    // Remove typical loader/mc version patterns like -neoforge-1.21.1-1.0.0
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

    async function resolveModIcons(files: BuildFileRow[]) {
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

        if (!missingHashes.length) return;

        // 2. Modrinth SHA1 Exact Lookup
        try {
            const res = await $fetch<Record<string, { project_id: string }>>(
                "https://api.modrinth.com/v2/version_files",
                {
                    method: "POST",
                    body: { hashes: missingHashes.slice(0, 100), algorithm: "sha1" },
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

        // 3. Fallback Search for remaining unresolved mods (by title query on Modrinth / CurseForge)
        const unresolved = modsToResolve.filter((f) => !icons.value[f.sha1]);
        const searchBatch = unresolved.slice(0, 15); // Limit search requests

        await Promise.allSettled(
            searchBatch.map(async (file) => {
                const title = cleanModTitle(file.path);
                if (!title || title.length < 3) return;

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

    return {
        icons,
        resolveModIcons,
    };
}
