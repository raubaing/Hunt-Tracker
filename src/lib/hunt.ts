// Données propres à Hunt: Showdown 1896 (faciles à compléter si de nouvelles maps sortent)
import type { Game, GameDetails, TeamSize } from "./types";

export const MAPS: { name: string; short: string }[] = [
  { name: "Stillwater Bayou", short: "Bayou" },
  { name: "Lawson Delta", short: "Delta" },
  { name: "DeSalle", short: "DeSalle" },
  { name: "Mammon's Gulch", short: "Gulch" },
];

export const TEAM_SIZES: { value: TeamSize; label: string }[] = [
  { value: 1, label: "Solo" },
  { value: 2, label: "Duo" },
  { value: 3, label: "Trio" },
];

export const STARS = [1, 2, 3, 4, 5, 6];

export const mapShort = (name?: string | null) => MAPS.find((m) => m.name === name)?.short ?? name ?? "";
export const teamLabel = (size?: number | null) => TEAM_SIZES.find((t) => t.value === size)?.label ?? "";

export function detailsOf(g: Game): GameDetails {
  return {
    teamSize: g.teamSize ?? null,
    map: g.map ?? null,
    extracted: g.extracted ?? null,
    bounty: g.bounty ?? null,
    mmrStars: g.mmrStars ?? null,
  };
}
