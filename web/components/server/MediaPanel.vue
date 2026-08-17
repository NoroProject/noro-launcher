<script setup lang="ts">
import type { ServerAssetKind } from "~/types/server-settings";

const props = defineProps<{
    serverId: string;
    iconUrl?: string | null;
    backgroundUrl?: string | null;
}>();
const emit = defineEmits<{ uploaded: [kind: ServerAssetKind, url: string] }>();

const auth = useAuth();
const { t } = useT();
const iconInput = ref<HTMLInputElement | null>(null);
const bgInput = ref<HTMLInputElement | null>(null);
const loading = reactive({ icon: false, background: false });
const error = ref<string | null>(null);

function endpoint(kind: ServerAssetKind) {
    return `/api/admin/servers/${props.serverId}/${kind === "icon" ? "icon" : "background"}`;
}

async function upload(kind: ServerAssetKind, file?: File) {
    if (!file) return;
    if (!file.type.startsWith("image/")) {
        error.value = "Only image files can be uploaded.";
        return;
    }
    loading[kind] = true;
    error.value = null;
    try {
        const res = await auth.upload<{ url: string }>(
            endpoint(kind),
            "image",
            file,
            undefined,
            "PUT",
        );
        emit("uploaded", kind, res.url);
    } catch (e) {
        error.value = humanError(e);
    } finally {
        loading[kind] = false;
    }
}

function pick(kind: ServerAssetKind, event: Event) {
    const input = event.target as HTMLInputElement;
    upload(kind, input.files?.[0]).finally(() => {
        input.value = "";
    });
}

function drop(kind: ServerAssetKind, event: DragEvent) {
    upload(kind, event.dataTransfer?.files[0]);
}
</script>

<template>
    <section class="noro-panel grid grid-rows-[auto_1fr] overflow-hidden">
        <div class="border-b border-[var(--noro-border)] p-5">
            <div class="flex items-center justify-between gap-4">
                <div>
                    <h2 class="text-base font-black text-white">{{ t('admin-media-title') }}</h2>
                    <p class="text-sm text-[var(--noro-muted)]">{{ t('admin-media-subtitle') }}</p>
                </div>
                <UIcon name="i-lucide-images" class="size-5 text-[var(--noro-blue)]" />
            </div>
        </div>

        <div class="flex min-h-0 flex-col gap-4 p-5">
            <UAlert
                v-if="error"
                color="error"
                variant="subtle"
                icon="i-lucide-circle-alert"
                :description="error"
            />

            <button
                type="button"
                class="noro-dropzone min-h-44 flex-1"
                @click="bgInput?.click()"
                @dragover.prevent
                @drop.prevent="drop('background', $event)"
            >
                <img
                    v-if="backgroundUrl"
                    :key="backgroundUrl"
                    :src="backgroundUrl"
                    alt=""
                    class="absolute inset-0 h-full w-full object-cover"
                >
                <span class="absolute inset-0 bg-[var(--noro-input)]/55" />
                <span class="relative z-10 grid justify-items-center gap-2">
                    <UIcon
                        :name="loading.background ? 'i-lucide-loader-circle' : 'i-lucide-image-up'"
                        class="size-8"
                        :class="{ 'animate-spin': loading.background }"
                    />
                    <span class="font-bold text-white">{{ t('admin-media-banner-upload') }}</span>
                    <span class="text-sm text-[var(--noro-muted)]">{{ t('admin-media-banner-hint') }}</span>
                </span>
            </button>
            <input ref="bgInput" class="sr-only" type="file" accept="image/*" @change="pick('background', $event)">

            <div class="grid gap-4 rounded border border-[var(--noro-border)] bg-[var(--noro-input)]/40 p-4 sm:grid-cols-[80px_1fr]">
                <button
                    type="button"
                    class="grid size-20 place-items-center overflow-hidden rounded border border-[var(--noro-border)] bg-[var(--noro-input)] text-[var(--noro-muted)]"
                    @click="iconInput?.click()"
                    @dragover.prevent
                    @drop.prevent="drop('icon', $event)"
                >
                    <img
                        v-if="iconUrl"
                        :key="iconUrl"
                        :src="iconUrl"
                        alt=""
                        class="h-full w-full object-cover"
                    >
                    <UIcon v-else name="i-lucide-image" class="size-7" />
                </button>
                <div class="grid content-center gap-2">
                    <div class="text-sm font-black uppercase text-white">{{ t('admin-media-icon') }}</div>
                    <AtomButton
                        :loading="loading.icon"
                        icon="i-lucide-upload"
                        variant="secondary"
                        @click="iconInput?.click()"
                    >
                        {{ t('admin-media-icon-upload') }}
                    </AtomButton>
                    <input ref="iconInput" class="sr-only" type="file" accept="image/*" @change="pick('icon', $event)">
                </div>
            </div>
        </div>
    </section>
</template>
