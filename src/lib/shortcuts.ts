// Conversion d'une touche pressée en raccourci compris par Tauri ("Ctrl+Shift+KeyK", "F8"…)

const MODIFIER_CODES = ["ControlLeft", "ControlRight", "ShiftLeft", "ShiftRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight"];

/** Renvoie null tant qu'on n'a appuyé que sur des modificateurs. */
export function eventToAccelerator(e: KeyboardEvent): string | null {
  if (MODIFIER_CODES.includes(e.code) || !e.code) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  parts.push(e.code); // KeyK, Digit1, F8, Numpad1, ArrowUp…
  return parts.join("+");
}

/** Affichage lisible : "Ctrl+Shift+KeyK" → "Ctrl + Shift + K" */
export function formatAccelerator(accel: string): string {
  if (!accel) return "Aucun";
  return accel
    .split("+")
    .map((p) =>
      p
        .replace(/^Key([A-Z])$/, "$1")
        .replace(/^Digit(\d)$/, "$1")
        .replace(/^Numpad(\d)$/, "Pavé num. $1")
        .replace(/^Arrow(\w+)$/, "Flèche $1"),
    )
    .join(" + ");
}
