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
/** Rouvre la dernière game validée (ses compteurs s'ajoutent à la game en cours) */
export const unvalidateGame = () => invoke<Session>("unvalidate_game");

// Correction des games déjà validées (n'importe quelle session)
export const updateGame = (sessionId: string, gameId: number, c: { kills: number; deaths: number; assists: number }) =>
  invoke<Session>("update_game", { sessionId, gameId, ...c });
export const deleteGame = (sessionId: string, gameId: number) =>
  invoke<Session>("delete_game", { sessionId, gameId });

// Options
export const getSettings = () => invoke<Settings>("get_settings");
export const setShortcuts = (shortcuts: Shortcuts) => invoke<void>("set_shortcuts", { shortcuts });
/** Mémorise la position de la barre du widget (pixels physiques) */
export const saveWidgetPosition = (x: number, y: number) =>
  invoke<void>("save_widget_position", { x: Math.round(x), y: Math.round(y) });
/** Coupe les raccourcis globaux pendant qu'on saisit une nouvelle touche */
export const pauseShortcuts = (paused: boolean) => invoke<void>("pause_shortcuts", { paused });

/** Une session a été créée, modifiée ou supprimée (toutes fenêtres confondues). */
export const onSessionsChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("sessions-changed", () => cb());

/** La session en cours a changé (ou son contenu). */
export const onActiveSessionChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("active-session-changed", () => cb());
