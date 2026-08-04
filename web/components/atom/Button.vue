<script setup lang="ts">
const props = withDefaults(defineProps<{
  variant?: 'primary' | 'secondary' | 'dark' | 'warning' | 'danger' | 'outline' | 'outline-blue' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  icon?: string
  iconRight?: string
  loading?: boolean
  disabled?: boolean
  to?: string
  type?: 'button' | 'submit'
  block?: boolean
}>(), {
  variant: 'secondary',
  size: 'md',
  type: 'button',
})

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const variantClass = computed(() => {
  const map: Record<string, string> = {
    primary: 'noro-btn-primary',
    secondary: 'noro-btn-secondary',
    dark: 'noro-btn-dark',
    warning: 'noro-btn-warning',
    danger: 'noro-btn-danger',
    outline: 'noro-btn-outline',
    'outline-blue': 'noro-btn-outline-blue',
    ghost: 'noro-btn-dark !bg-transparent hover:!bg-white/5',
  }
  return map[props.variant] || 'noro-btn-secondary'
})

const sizeClass = computed(() => {
  if (props.size === 'sm') return 'noro-btn-sm'
  if (props.size === 'lg') return 'noro-btn-lg'
  return ''
})

const isDisabled = computed(() => props.disabled || props.loading)

function handleClick(e: MouseEvent) {
  if (isDisabled.value) {
    e.preventDefault()
    return
  }
  emit('click', e)
}
</script>

<template>
  <NuxtLink
    v-if="to"
    :to="to"
    :class="['noro-btn', variantClass, sizeClass, { 'w-full': block }]"
    :aria-disabled="isDisabled"
    @click="handleClick"
  >
    <UIcon v-if="loading" name="i-lucide-loader-circle" class="size-4 shrink-0 animate-spin" />
    <UIcon v-else-if="icon" :name="icon" class="size-4 shrink-0" />
    <span class="truncate"><slot /></span>
    <UIcon v-if="iconRight && !loading" :name="iconRight" class="size-4 shrink-0" />
  </NuxtLink>

  <button
    v-else
    :type="type"
    :class="['noro-btn', variantClass, sizeClass, { 'w-full': block }]"
    :disabled="isDisabled"
    @click="handleClick"
  >
    <UIcon v-if="loading" name="i-lucide-loader-circle" class="size-4 shrink-0 animate-spin" />
    <UIcon v-else-if="icon" :name="icon" class="size-4 shrink-0" />
    <span class="truncate"><slot /></span>
    <UIcon v-if="iconRight && !loading" :name="iconRight" class="size-4 shrink-0" />
  </button>
</template>

