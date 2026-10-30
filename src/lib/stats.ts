import type { Session } from "./types";

export interface Totals {
  kills: number;
  deaths: number;
  assists: number;
  games: number;
}

/** Totaux de la session : games validées + game en cours */
export function sessionTotals(session: Session): Totals {
  const t = session.games.reduce(
    (t, g) => ({ kills: t.kills + g.kills, deaths: t.deaths + g.deaths, assists: t.assists + g.assists }),
    { ...session.current },
  );
  return { ...t, games: session.games.length };
}

/** KDA = (K + A) / D ; si 0 mort, KDA = K + A */
export function kda(t: { kills: number; deaths: number; assists: number }): number {
  return (t.kills + t.assists) / Math.max(t.deaths, 1);
}

export const formatKda = (value: number) => value.toFixed(2);
