const pad = (n: number) => String(n).padStart(2, "0");

/** "jeudi 30 octobre 2026 à 21:05" */
export function formatStart(iso: string): string {
  const d = new Date(iso);
  const date = d.toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long", year: "numeric" });
  return `${date} à ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
