<script setup lang="ts">
// Язык страницы — выбранный, а не вшитый в конфиг: по нему браузер решает,
// предлагать ли перевод, и как читать текст экранным диктором.
const { locale } = useT()
useHead({ htmlAttrs: { lang: locale } })
</script>

<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
