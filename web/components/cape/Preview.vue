<script setup lang="ts">
const props = defineProps<{ url: string; alt?: string }>();
const masterUrl = useRuntimeConfig().public.masterUrl;

const renderUrl = computed(() => {
    if (!props.url) return '';
    return `${masterUrl}/api/textures/renders/cape?url=${encodeURIComponent(props.url)}&scale=10`;
});
</script>

<template>
    <div class="cape-preview flex items-center justify-center overflow-hidden aspect-[10/16] rounded bg-[var(--noro-bg-deep)]">
        <img
            :src="renderUrl"
            :alt="alt || 'Cape'"
            class="h-full w-auto object-contain [image-rendering:pixelated]"
        >
    </div>
</template>
