export function plural(n, word) {
  return `${n} ${word}${n > 1 ? "s" : ""}`;
}

export function formatDay(iso) {
  const [y, m, d] = iso.split("-").map(Number);
  const year = y === new Date().getFullYear() ? undefined : "numeric";
  return new Date(y, m - 1, d).toLocaleDateString("fr-FR", { weekday: "short", day: "numeric", month: "short", year });
}

export function formatTime(seconds) {
  return new Date(seconds * 1000).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
}

export function formatDuration(seconds) {
  const minutes = Math.round(seconds / 60);
  if (minutes < 1) return "moins d'une minute";
  if (minutes < 60) return `${minutes} min`;
  return `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, "0")}`;
}
