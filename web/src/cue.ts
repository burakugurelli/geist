const cueNames = ["press", "success", "error"] as const;

export type CueName = (typeof cueNames)[number];

const supportedCues = new Set<string>(cueNames);

export function isCueName(value: unknown): value is CueName {
  return typeof value === "string" && supportedCues.has(value);
}
