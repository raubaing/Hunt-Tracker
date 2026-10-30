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

export interface Session {
  id: string; // ex. "2026-10-30_210512" (sert aussi de nom de fichier)
  name: string; // ex. "Session du 30/10/26"
  startedAt: string;
  endedAt?: string;
  note?: string;
  games: Game[];
}
