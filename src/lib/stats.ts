import type { Game } from "./types";

export interface Totals {
  kills: number;
  deaths: number;
  assists: number;
  games: number;
}

export function totals(games: Game[]): Totals {
  return games.reduce(
    (t, g) => ({
      kills: t.kills + g.kills,
      deaths: t.deaths + g.deaths,
      assists: t.assists + g.assists,
      games: t.games + 1,
    }),
    { kills: 0, deaths: 0, assists: 0, games: 0 },
  );
}

/** KDA = (K + A) / D ; si 0 mort, KDA = K + A */
export function kda(t: { kills: number; deaths: number; assists: number }): number {
  return (t.kills + t.assists) / Math.max(t.deaths, 1);
}

export const formatKda = (value: number) => value.toFixed(2);
