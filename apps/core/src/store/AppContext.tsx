// Application store: the signed-in actor, permission gating, and toasts.

import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { api, ApiError } from "../lib/api";
import type { MeDto } from "../lib/types";
import { I, type IconName } from "../components/icons";

export interface ToastItem {
  id: string;
  msg: React.ReactNode;
  icon?: IconName;
}

interface Store {
  me: MeDto | null;
  ready: boolean;
  accent: string;
  setAccent: (hex: string) => void;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshMe: () => Promise<void>;
  can: (action: string) => boolean;
  toast: (msg: React.ReactNode, icon?: IconName) => void;
  toasts: ToastItem[];
}

const Ctx = createContext<Store | null>(null);

export function useStore(): Store {
  const s = useContext(Ctx);
  if (!s) throw new Error("useStore must be used within AppProvider");
  return s;
}

let toastSeq = 0;

export function AppProvider({ children }: { children: React.ReactNode }) {
  const [me, setMe] = useState<MeDto | null>(null);
  const [ready, setReady] = useState(false);
  const [accent, setAccent] = useState("#D6B982");
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const toast = useCallback((msg: React.ReactNode, icon?: IconName) => {
    const id = `t${toastSeq++}`;
    setToasts((ts) => [...ts, { id, msg, icon }]);
    window.setTimeout(() => setToasts((ts) => ts.filter((t) => t.id !== id)), 4200);
  }, []);

  const refreshMe = useCallback(async () => {
    try {
      setMe(await api.me());
    } catch (e) {
      if (e instanceof ApiError && e.status === 401) setMe(null);
    }
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    setMe(await api.login(email, password));
  }, []);

  const logout = useCallback(async () => {
    await api.logout().catch(() => {});
    setMe(null);
  }, []);

  // Initial session check.
  useEffect(() => {
    void refreshMe().finally(() => setReady(true));
  }, [refreshMe]);

  // Best-effort: sync the brand accent from org settings once signed in.
  useEffect(() => {
    if (!me) return;
    api.getOrg().then((o) => setAccent(o.branding.accentColor)).catch(() => {});
  }, [me]);

  const can = useCallback((action: string) => !!me?.permissions.includes(action), [me]);

  const value = useMemo<Store>(
    () => ({ me, ready, accent, setAccent, login, logout, refreshMe, can, toast, toasts }),
    [me, ready, accent, login, logout, refreshMe, can, toast, toasts],
  );

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function ToastHost() {
  const { toasts } = useStore();
  return (
    <div className="toasts">
      {toasts.map((t) => {
        const IconCmp = t.icon ? I[t.icon] : I.check;
        return (
          <div className="toast" key={t.id}>
            <IconCmp />
            <div className="tmsg">{t.msg}</div>
          </div>
        );
      })}
    </div>
  );
}
