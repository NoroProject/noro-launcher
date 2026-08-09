<script setup lang="ts">
const props = withDefaults(defineProps<{
  skinUrl?: string
  preset?: string
  mode?: 'body' | 'head' | 'cube' | 'bust'
  scale?: number
  overlay?: boolean
}>(), {
  mode: 'body',
  scale: 10,
  overlay: true
})

const masterUrl = useRuntimeConfig().public.masterUrl

const renderUrl = computed(() => {
  const params = new URLSearchParams()
  params.set('mode', props.mode)

  if (props.preset) {
    params.set('preset', props.preset)
  } else if (props.skinUrl) {
    const match = props.skinUrl.match(/\/presets\/([a-zA-Z0-9_-]+)/)
    if (match) {
      params.set('preset', match[1])
    } else {
      params.set('url', props.skinUrl)
    }
  }

  if (props.scale) params.set('scale', String(props.scale))
  if (props.overlay === false) params.set('overlay', 'false')

  return `${masterUrl}/api/textures/renders?${params.toString()}`
})
</script>

<template>
  <div class="relative flex items-center justify-center overflow-hidden h-[125px] w-[100px] pointer-events-none select-none">
    <img :src="renderUrl" class="h-full w-auto object-contain drop-shadow" alt="Skin Render">
  </div>
</template>
