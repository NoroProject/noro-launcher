import type { ComputedRef } from "vue";
import type { Role } from "~/types/api";

type AuthApi = ReturnType<typeof useAuth>;

const adminPerms = ["*", "noro.admin.*", "noro.admin.servers"];

export function useAdminServerRoles(
  auth: AuthApi,
  id: ComputedRef<string>,
) {
  const notify = useNotify();
  const rolesData = useAsyncData(
    "admin-roles-list",
    () => auth.request<Role[]>("/api/admin/roles"),
    { default: () => [] },
  );
  const roles = rolesData.data;
  const togglingRole = ref<string | null>(null);

  function hasAccess(role: Role) {
    if (role.permissions.some((permission) => adminPerms.includes(permission))) {
      return true;
    }
    return role.permissions.includes(`noro.server.${id.value}.join`);
  }

  async function toggleRoleAccess(role: Role) {
    if (togglingRole.value || (hasAccess(role) && role.permissions.includes("*"))) {
      return;
    }
    togglingRole.value = role.id;
    const perm = `noro.server.${id.value}.join`;
    const has = role.permissions.includes(perm);
    try {
      if (has) {
        await auth.request(
          `/api/admin/roles/${role.id}/permissions/${encodeURIComponent(perm)}`,
          { method: "DELETE" },
        );
        role.permissions = role.permissions.filter((item) => item !== perm);
      } else {
        await auth.request(`/api/admin/roles/${role.id}/permissions`, {
          method: "POST",
          body: { permission: perm },
        });
        role.permissions.push(perm);
      }
      notify.ok()
    } catch (e) {
      notify.fail(e)
    } finally {
      togglingRole.value = null;
    }
  }

  return { roles, togglingRole, hasAccess, toggleRoleAccess, rolesData };
}
