export async function useAdminServerEditor() {
  const base = useAdminServerBase();
  const builds = useAdminServerBuilds(
    base.auth,
    base.id,
    base.loader,
    base.loadLoader,
  );
  const roles = useAdminServerRoles(base.auth, base.id);

  await base.auth.loadMe();
  await Promise.all([
    base.serversData,
    builds.buildsData,
    roles.rolesData,
  ]);

  return {
    id: base.id,
    server: base.server,
    form: base.form,
    activeTab: base.activeTab,
    minecraft: base.minecraft,
    modloaders: base.modloaders,
    loading: base.loading,
    save: base.save,
    remove: base.remove,
    saving: base.saving,
    deleting: base.deleting,
    applyAsset: base.applyAsset,
    ...builds,
    ...roles,
  };
}
