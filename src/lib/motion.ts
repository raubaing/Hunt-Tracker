// Réglages communs des animations

/** L'utilisateur a demandé à Windows de réduire les animations */
export const reducedMotion =
  typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Durée en ms, ramenée à 0 si les animations sont réduites */
export const dur = (ms: number) => (reducedMotion ? 0 : ms);

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
