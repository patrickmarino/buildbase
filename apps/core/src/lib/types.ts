// TypeScript mirrors of the API DTOs (camelCase JSON from core-web).

export type PermissionState = "allow" | "scope" | "deny";
export type UserStatus = "active" | "invited" | "deactivated";
export type ApiKeyStatus = "active" | "revoked";

export interface UserDto {
  id: string;
  name: string;
  email: string;
  roleId: string;
  roleKey: string;
  roleName: string;
  status: UserStatus;
  scope: string | null;
  lastActive: string | null;
  createdAt: string;
}

export interface RoleDto {
  id: string;
  key: string;
  name: string;
  isColumn: boolean;
  custom: boolean;
  protected: boolean;
  inherits: string;
  rank: number;
}

export interface ActionDto {
  key: string;
  label: string;
}
export interface GroupDto {
  category: string;
  label: string;
  actions: ActionDto[];
}
export interface CellDto {
  actionKey: string;
  roleId: string;
  state: PermissionState;
  locked: boolean;
}
export interface MatrixDto {
  groups: GroupDto[];
  columns: RoleDto[];
  cells: CellDto[];
}
export interface CellResultDto {
  actionKey: string;
  roleId: string;
  state: PermissionState;
}

export interface AuditDto {
  id: string;
  ts: string;
  actor: { name: string; role: string };
  action: string;
  category: string;
  target: string | null;
  before: string | null;
  after: string | null;
  ip: string | null;
}

export interface ApiKeyDto {
  id: string;
  name: string;
  prefix: string;
  scopes: string[];
  status: ApiKeyStatus;
  created: string;
  lastUsed: string | null;
}
export interface CreatedKeyDto {
  key: ApiKeyDto;
  token: string;
}

export interface MfaDto {
  enabled: boolean;
  method: "totp" | "sms" | "webauthn";
  enforce: "all" | "admins" | "none";
}
export interface PasswordPolicyDto {
  minLength: number;
  requireNumber: boolean;
  requireSymbol: boolean;
  rotationDays: number | null;
}
export interface SsoDto {
  enabled: boolean;
  provider: "saml" | "oidc";
  url: string | null;
}
export interface OrgDto {
  id: string;
  name: string;
  domain: string | null;
  branding: { accentColor: string };
  mfa: MfaDto;
  passwordPolicy: PasswordPolicyDto;
  sso: SsoDto;
  ownerId: string;
  pendingOwnerId: string | null;
}

export interface UpdateOrgPatch {
  name?: string;
  domain?: string;
  accentColor?: string;
  mfa?: MfaDto;
  passwordPolicy?: PasswordPolicyDto;
  sso?: SsoDto;
}

export interface MeDto {
  user: UserDto;
  permissions: string[];
}

export interface ApiErrorBody {
  error: string;
  message: string;
}
