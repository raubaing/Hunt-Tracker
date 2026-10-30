// Modèle de données (à compléter lors du développement des fonctionnalités)

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
  timestamp: string; // ISO 8601
}

export interface Session {
  id: string; // ex. "2026-10-30_2105"
  startedAt: string;
  endedAt?: string;
  note?: string;
  games: Game[];
}
