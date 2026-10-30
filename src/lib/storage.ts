// Accès aux données (géré côté Rust, dans %APPDATA%\com.robinnaud.hunttracker)
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Session, Settings, Shortcuts, Stat } from "./types";

// Sessions
export const listSessions = () => invoke<Session[]>("list_sessions");
/** Crée la session du jour et en fait la session en cours */
export const createSession = () => invoke<Session>("create_session");
export const saveSession = (session: Session) => invoke<void>("save_session", { session });
export const deleteSession = (id: string) => invoke<void>("delete_session", { id });
export const getActiveSession = () => invoke<Session | null>("get_active_session");
export const setActiveSession = (id: string | null) => invoke<void>("set_active_session", { id });

// Game en cours (une session est créée automatiquement s'il n'y en a pas)
export const addStat = (stat: Stat, delta: 1 | -1) => invoke<Session>("add_stat", { stat, delta });
export const validateGame = () => invoke<Session>("validate_game");

// Options
export const getSettings = () => invoke<Settings>("get_settings");
export const setShortcuts = (shortcuts: Shortcuts) => invoke<void>("set_shortcuts", { shortcuts });
/** Coupe les raccourcis globaux pendant qu'on saisit une nouvelle touche */
export const pauseShortcuts = (paused: boolean) => invoke<void>("pause_shortcuts", { paused });

/** Une session a été créée, modifiée ou supprimée (toutes fenêtres confondues). */
export const onSessionsChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("sessions-changed", () => cb());

/** La session en cours a changé (ou son contenu). */
export const onActiveSessionChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("active-session-changed", () => cb());
