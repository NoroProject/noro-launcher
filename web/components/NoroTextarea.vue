<script setup lang="ts">
/**
 * Многострочное поле, растущее под содержимое.
 *
 * Не `UTextarea`: тот приходит со светлой темой Nuxt UI и выпадал белым пятном
 * на тёмной странице — та же причина, по которой в проекте есть `NoroModal` и
 * `NoroNote` вместо `UModal` и `UAlert`.
 *
 * Автовысота здесь не украшение: шаблоны наказаний бывают и в три строки, и в
 * двадцать, а полоса прокрутки внутри поля прячет ровно ту часть текста,
 * которую правят.
 */
const model = defineModel<string>({ required: true })
withDefaults(defineProps<{ rows?: number; disabled?: boolean }>(), { rows: 4 })

const el = ref<HTMLTextAreaElement | null>(null)

function fit() {
    const node = el.value
    if (!node) return
    // Сброс перед замером обязателен: `scrollHeight` не уменьшается сам, и без
    // этого поле только росло бы, даже когда текст стёрли.
    node.style.height = 'auto'
    node.style.height = `${node.scrollHeight}px`
}

// Правка извне — кнопка «вернуть встроенный», смена языка — тоже меняет высоту.
watch(model, () => nextTick(fit))
onMounted(fit)
</script>

<template>
    <textarea
        ref="el"
        v-model="model"
        :rows="rows"
        :disabled="disabled"
        spellcheck="false"
        class="noro-input w-full resize-y overflow-hidden font-mono text-xs leading-6"
        @input="fit"
    />
</template>
