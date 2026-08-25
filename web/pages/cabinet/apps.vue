<script setup lang="ts">
/**
 * Приложения в кабинете: чужие, которым игрок открыл доступ, и свои, которые он
 * сам написал. Рядом, потому что вопрос один — «что связано с моим аккаунтом».
 */

const auth = useAuth()
const { t } = useT()
await auth.loadMe()
</script>

<template>
  <NoroShell :title="t('cabinet-apps-title')" :subtitle="t('cabinet-apps-subtitle')">
    <div class="grid gap-4">
      <CabinetAuthorizedApps />
      <CabinetMyApps />
    </div>
  </NoroShell>
</template>
