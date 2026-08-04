<script setup lang="ts">
const props = defineProps<{
    skinUrl?: string | null;
    capeUrl?: string | null;
}>();

const canvas = ref<HTMLCanvasElement | null>(null);
const frame = ref<HTMLElement | null>(null);
const isReady = ref(false);
const error = ref<string | null>(null);
let viewer: import("skinview3d").SkinViewer | null = null;
let observer: ResizeObserver | null = null;

function resize() {
    if (!viewer || !frame.value) return;
    const rect = frame.value.getBoundingClientRect();
    viewer.setSize(Math.max(240, Math.floor(rect.width)), 420);
}

async function loadTextures() {
    if (!viewer) return;
    try {
        if (props.skinUrl) {
            await viewer.loadSkin(props.skinUrl, { model: "auto-detect" });
        } else {
            viewer.loadSkin(null);
        }
        if (props.capeUrl) {
            await viewer.loadCape(props.capeUrl, { backEquipment: "cape" });
        } else {
            viewer.loadCape(null);
        }
        error.value = null;
    } catch {
        error.value = "Preview texture failed to load";
    }
}

onMounted(async () => {
    if (!canvas.value || !frame.value) return;
    const skinview3d = await import("skinview3d");
    viewer = new skinview3d.SkinViewer({
        canvas: canvas.value,
        width: 320,
        height: 420,
        enableControls: true,
        zoom: 0.88,
    });
    viewer.autoRotate = true;
    viewer.autoRotateSpeed = 0.8;
    viewer.animation = new skinview3d.IdleAnimation();
    observer = new ResizeObserver(resize);
    observer.observe(frame.value);
    resize();
    await loadTextures();
    isReady.value = true;
});

watch(() => [props.skinUrl, props.capeUrl], () => void loadTextures());

onBeforeUnmount(() => {
    observer?.disconnect();
    viewer?.dispose();
});
</script>

<template>
    <div ref="frame" class="relative min-h-[420px] overflow-hidden rounded-lg bg-[var(--noro-bg-deep)]">
        <canvas ref="canvas" class="block h-[420px] w-full" />
        <div v-if="!isReady" class="absolute inset-0 grid place-items-center bg-[var(--noro-bg-deep)]">
            <UIcon name="i-lucide-loader-2" class="size-8 animate-spin text-[var(--noro-blue)]" />
        </div>
        <div v-if="!skinUrl" class="absolute inset-x-4 bottom-4 rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-panel)] p-3 text-center">
            <p class="text-sm font-bold text-[var(--noro-muted)]">Default skin — upload your own to replace it.</p>
        </div>
        <div v-if="error" class="absolute inset-x-4 top-4 rounded-lg bg-[var(--noro-danger)] p-3">
            <p class="text-sm font-bold text-[var(--noro-white)]">{{ error }}</p>
        </div>
    </div>
</template>
