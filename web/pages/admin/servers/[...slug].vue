<script setup lang="ts">
const route = useRoute();
const targetPath = route.path.replace(/^\/admin\/servers/, "/admin/clients");
await navigateTo(targetPath, { replace: true });
</script>
<template>
  <div />
</template>
