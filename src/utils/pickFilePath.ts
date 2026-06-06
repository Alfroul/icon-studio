export function pickFilePath(selected: string | string[] | null): string | null {
  if (selected == null) return null;
  if (typeof selected === "string") return selected;
  if (Array.isArray(selected) && selected.length > 0) return selected[0];
  return null;
}
