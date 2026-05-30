// Typed API client. All requests send the session cookie (`credentials:
// "include"`). On a non-2xx response it throws an `ApiError` carrying the
// backend's stable `code`, so callers can branch on it.

import type {
  ApiErrorBody,
  ApiKeyDto,
  AuditDto,
  CellResultDto,
  CreatedKeyDto,
  MatrixDto,
  MeDto,
  OrgDto,
  RoleDto,
  UpdateOrgPatch,
  UserDto,
} from "./types";

const BASE = import.meta.env.VITE_API_BASE ?? "";

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}/api${path}`, {
    method,
    credentials: "include",
    headers: body !== undefined ? { "Content-Type": "application/json" } : undefined,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    let code = "error";
    let message = res.statusText;
    try {
      const data = (await res.json()) as ApiErrorBody;
      code = data.error ?? code;
      message = data.message ?? message;
    } catch {
      /* non-JSON error body */
    }
    throw new ApiError(res.status, code, message);
  }
  if (res.status === 204) return undefined as T;
  const ct = res.headers.get("content-type") ?? "";
  return (ct.includes("application/json") ? await res.json() : await res.text()) as T;
}

export interface ListUsersParams {
  role?: string;
  status?: string;
  q?: string;
}
export interface AuditParams {
  q?: string;
  category?: string;
  page?: number;
}

function qs(params: Record<string, string | number | undefined>): string {
  const entries = Object.entries(params).filter(([, v]) => v !== undefined && v !== "");
  if (entries.length === 0) return "";
  return "?" + entries.map(([k, v]) => `${k}=${encodeURIComponent(String(v))}`).join("&");
}

export const api = {
  // ── auth ──
  login: (email: string, password: string) =>
    request<MeDto>("POST", "/auth/login", { email, password }),
  logout: () => request<void>("POST", "/auth/logout"),
  me: () => request<MeDto>("GET", "/auth/me"),

  // ── users ──
  listUsers: (p: ListUsersParams = {}) =>
    request<UserDto[]>("GET", `/users${qs(p as Record<string, string | undefined>)}`),
  invite: (email: string, role: string, scope?: string) =>
    request<UserDto>("POST", "/users/invite", { email, role, scope }),
  createUser: (input: { name: string; email: string; role: string; scope?: string; password: string }) =>
    request<UserDto>("POST", "/users", input),
  changeRole: (userId: string, roleId: string) =>
    request<UserDto>("PATCH", `/users/${userId}/role`, { roleId }),
  setStatus: (userId: string, status: string) =>
    request<UserDto>("PATCH", `/users/${userId}/status`, { status }),

  // ── roles & permissions ──
  listRoles: () => request<RoleDto[]>("GET", "/roles"),
  createRole: (name: string, baseRoleId: string) =>
    request<RoleDto>("POST", "/roles", { name, baseRoleId }),
  matrix: () => request<MatrixDto>("GET", "/permissions/matrix"),
  cycleCell: (actionKey: string, roleId: string) =>
    request<CellResultDto>("PATCH", "/permissions/matrix/cell", { actionKey, roleId }),

  // ── org ──
  getOrg: () => request<OrgDto>("GET", "/org"),
  updateOrg: (patch: UpdateOrgPatch) => request<OrgDto>("PATCH", "/org", patch),
  transferOwnership: (targetUserId: string) =>
    request<void>("POST", "/org/transfer-ownership", { targetUserId }),
  deleteOrg: () => request<void>("DELETE", "/org"),

  // ── audit ──
  audit: (p: AuditParams = {}) => request<AuditDto[]>("GET", `/audit${qs(p as Record<string, string | number | undefined>)}`),
  auditExportUrl: (p: AuditParams = {}) =>
    `${BASE}/api/audit/export${qs(p as Record<string, string | number | undefined>)}`,

  // ── keys ──
  listKeys: () => request<ApiKeyDto[]>("GET", "/keys"),
  createKey: (name: string, scopes: string[]) =>
    request<CreatedKeyDto>("POST", "/keys", { name, scopes }),
  revokeKey: (id: string) => request<void>("POST", `/keys/${id}/revoke`),
};
