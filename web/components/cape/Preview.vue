<script setup lang="ts">
/**
 * Лицевая сторона плаща из текстуры Minecraft.
 *
 * В файле лежит развёртка целиком, и показывать её как есть бессмысленно —
 * видно кашу из пикселей вместо плаща. Лицо занимает прямоугольник 10×16,
 * начиная с (1,1) при раскладке 64×32; проценты берутся от размеров текстуры,
 * поэтому работают и для 512×256 — пропорции у обеих одинаковые.
 */
defineProps<{ url: string; alt?: string }>();

// 64/10 и 32/16 — во столько раз текстура шире и выше вырезаемого куска.
const SCALE_X = "640%";
const SCALE_Y = "200%";
// Отступ в один текстурный пиксель: 1/10 ширины куска и 1/16 его высоты.
const OFFSET_X = "-10%";
const OFFSET_Y = "-6.25%";
</script>

<template>
    <div class="cape-preview">
        <img
            :src="url"
            :alt="alt"
            :style="{ width: SCALE_X, height: SCALE_Y, left: OFFSET_X, top: OFFSET_Y }"
        >
    </div>
</template>

<style scoped>
.cape-preview {
    position: relative;
    aspect-ratio: 10 / 16;
    overflow: hidden;
    border-radius: var(--noro-r-sm);
    background: var(--noro-bg-deep);
}

.cape-preview img {
    position: absolute;
    max-width: none;
    /* Плащи — пиксель-арт: сглаживание превращает их в мыло. */
    image-rendering: pixelated;
}
</style>
