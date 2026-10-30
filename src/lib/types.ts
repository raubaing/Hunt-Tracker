// Modèle de données (miroir des structs Rust dans src-tauri/src/lib.rs)

export type GameMode = "bounty" | "clash";
export type TeamSize = 1 | 2 | 3;

export interface Game {
  id: number;
  kills: number;
  deaths: number;
  assists: number;
  mode?: GameMode;
  teamSize?: TeamSize;
  map?: string;
  extracted?: boolean;
  bounty?: boolean;
  mmrStars?: number;
  timestamp: string; // date locale ISO, ex. "2026-10-30T21:32:00"
}

/** Compteurs de la game en cours, pas encore validée */
export interface Counters {
  kills: number;
  deaths: number;
  assists: number;
}

export type Stat = "kills" | "deaths" | "assists";

export interface Session {
  id: string; // ex. "2026-10-30_210512" (sert aussi de nom de fichier)
  name: string; // ex. "Session du 30/10/26"
  startedAt: string;
  endedAt?: string;
  note?: string;
  games: Game[];
  current: Counters;
  /** Dernière action sur la session (compteur, validation, sélection) */
  lastActivity?: string;
  /** Coéquipiers de la session */
  teammates?: string[];
}

/** Raccourcis clavier globaux, au format "F8", "Ctrl+Shift+K"… ("" = désactivé) */
export interface Shortcuts {
  kill: string;
  assist: string;
  death: string;
  validate: string;
  /** Plier / déplier le panneau */
  toggle: string;
}

/** Infos Hunt d'une game, renseignées après la validation (toutes facultatives) */
export interface GameDetails {
  teamSize?: TeamSize | null;
  map?: string | null;
  extracted?: boolean | null;
  bounty?: boolean | null;
  mmrStars?: number | null;
}

export interface Preferences {
  /** Proposer les infos de la game juste après la validation */
  showDetailsPrompt: boolean;
  /** Position du widget verrouillée */
  locked: boolean;
}

export interface Settings {
  shortcuts: Shortcuts;
  preferences: Preferences;
  /** Taille mémorisée du panneau déplié (null = taille par défaut) */
  panelSize: { w: number; h: number } | null;
  shortcutError: string | null;
}

/** Onglets du panneau déplié */
export type Tab = "sessions" | "stats" | "options";
