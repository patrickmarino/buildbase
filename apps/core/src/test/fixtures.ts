// Shared fixtures for component/page tests.

import type { MatrixDto, MeDto, OrgDto, RoleDto } from "../lib/types";

export const meOwner: MeDto = {
  user: {
    id: "u-owner",
    name: "Elena Marchetti",
    email: "elena@madespace.co",
    roleId: "r-owner",
    roleKey: "owner",
    roleName: "Owner",
    status: "active",
    scope: "Studio",
    lastActive: "2026-05-30T09:00:00Z",
    createdAt: "2026-05-01T09:00:00Z",
  },
  permissions: ["roles.matrix", "roles.edit", "users.view", "users.invite", "org.delete", "keys.manage", "keys.scope"],
};

export const roles: RoleDto[] = [
  { id: "r-owner", key: "owner", name: "Owner", isColumn: true, custom: false, protected: true, inherits: "Admin", rank: 60 },
  { id: "r-admin", key: "admin", name: "Admin", isColumn: true, custom: false, protected: false, inherits: "Manager", rank: 50 },
  { id: "r-viewer", key: "viewer", name: "Viewer", isColumn: true, custom: false, protected: false, inherits: "—", rank: 20 },
  { id: "r-guest", key: "guest", name: "Guest", isColumn: false, custom: false, protected: false, inherits: "—", rank: 10 },
];

export const matrix: MatrixDto = {
  groups: [
    { category: "users", label: "Users", actions: [{ key: "users.invite", label: "Invite user" }] },
    { category: "org", label: "Organization settings", actions: [{ key: "org.delete", label: "Delete organization" }] },
  ],
  columns: [roles[0], roles[1], roles[2]],
  cells: [
    { actionKey: "users.invite", roleId: "r-owner", state: "allow", locked: true },
    { actionKey: "users.invite", roleId: "r-admin", state: "allow", locked: false },
    { actionKey: "users.invite", roleId: "r-viewer", state: "deny", locked: false },
    { actionKey: "org.delete", roleId: "r-owner", state: "allow", locked: true },
    { actionKey: "org.delete", roleId: "r-admin", state: "deny", locked: true },
    { actionKey: "org.delete", roleId: "r-viewer", state: "deny", locked: true },
  ],
};

export const org: OrgDto = {
  id: "o-1",
  name: "MadeSpace Studio",
  domain: "madespace.co",
  branding: { accentColor: "#D6B982" },
  mfa: { enabled: true, method: "totp", enforce: "admins" },
  passwordPolicy: { minLength: 12, requireNumber: true, requireSymbol: true, rotationDays: 90 },
  sso: { enabled: false, provider: "saml", url: null },
  ownerId: "u-owner",
  pendingOwnerId: null,
};
