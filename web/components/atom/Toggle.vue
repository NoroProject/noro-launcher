<script setup lang="ts">
// Переключатель серверного флага. Состояние приходит пропом и никогда не
// меняется локально: пока запрос не подтверждён, тумблер обязан показывать то,
// что реально лежит на сервере, — иначе UI врёт при упавшем запросе.
withDefaults(
    defineProps<{
        modelValue: boolean;
        label?: string;
        disabled?: boolean;
        loading?: boolean;
    }>(),
    { disabled: false, loading: false },
);
defineEmits<{ "update:modelValue": [value: boolean] }>();
</script>

<template>
    <button
        type="button"
        role="switch"
        :aria-checked="modelValue"
        :aria-label="label"
        :disabled="disabled || loading"
        class="noro-toggle"
        :class="{ 'is-on': modelValue, 'is-busy': loading }"
        @click="$emit('update:modelValue', !modelValue)"
    >
        <span class="noro-toggle-track">
            <span class="noro-toggle-thumb" />
        </span>
        <span v-if="label" class="noro-toggle-label">{{ label }}</span>
    </button>
</template>

<style scoped>
.noro-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    background: none;
    border: 0;
    padding: 0;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    color: var(--noro-muted);
    transition: color 0.15s ease;
}

.noro-toggle:hover:not(:disabled) {
    color: var(--noro-text);
}

.noro-toggle:disabled {
    cursor: not-allowed;
}

.noro-toggle.is-busy {
    opacity: 0.6;
}

.noro-toggle-track {
    position: relative;
    display: block;
    width: 44px;
    height: 24px;
    flex: none;
    border-radius: var(--noro-r-lg);
    background: var(--noro-input);
    border: 1px solid var(--noro-border);
    transition:
        background 0.15s ease,
        border-color 0.15s ease;
}

.noro-toggle:focus-visible .noro-toggle-track {
    outline: 2px solid var(--noro-magenta);
    outline-offset: 2px;
}

.noro-toggle-thumb {
    position: absolute;
    top: 3px;
    left: 4px;
    width: 16px;
    height: 16px;
    border-radius: var(--noro-r-sm);
    background: var(--noro-muted);
    transition:
        transform 0.15s ease,
        background 0.15s ease;
}

.noro-toggle.is-on .noro-toggle-track {
    background: color-mix(in srgb, var(--noro-green) 28%, var(--noro-input));
    border-color: var(--noro-green);
}

.noro-toggle.is-on .noro-toggle-thumb {
    background: var(--noro-green);
    transform: translateX(20px);
}

.noro-toggle.is-on .noro-toggle-label {
    color: var(--noro-text);
}
</style>
