export type ParsedCue = { scene: string; name: string };

export function parseCues(text: string): ParsedCue[] {
  return text.split('\n').map((line) => line.trim()).filter(Boolean).map((line) => {
    const parts = line.split('|');
    if (parts.length < 2) return { scene: 'Scene', name: line.slice(0, 100).trim() };
    return { scene: parts.shift()!.trim().slice(0, 60), name: parts.join('|').trim().slice(0, 100) };
  }).filter((cue) => cue.scene && cue.name);
}

export function csvEscape(value: unknown): string {
  const text = String(value ?? '');
  return `"${text.replaceAll('"', '""')}"`;
}

export function formatClock(iso: string): string {
  return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(new Date(iso));
}
