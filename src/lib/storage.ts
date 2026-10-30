// Accès aux sessions stockées en JSON (géré côté Rust, dans %APPDATA%\com.robinnaud.hunttracker)
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Session } from "./types";

export const listSessions = () => invoke<Session[]>("list_sessions");
export const saveSession = (session: Session) => invoke<void>("save_session", { session });
export const deleteSession = (id: string) => invoke<void>("delete_session", { id });
export const getActiveSession = () => invoke<Session | null>("get_active_session");
export const setActiveSession = (id: string | null) => invoke<void>("set_active_session", { id });

/** Une session a été créée, modifiée ou supprimée (toutes fenêtres confondues). */
export const onSessionsChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("sessions-changed", () => cb());

/** La session en cours a changé (ou son contenu). */
export const onActiveSessionChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("active-session-changed", () => cb());
