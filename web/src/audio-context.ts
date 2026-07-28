type WebKitWindow = Window & {
  webkitAudioContext?: typeof AudioContext;
};

export function createAudioContext(): AudioContext | null {
  if (typeof window === "undefined") {
    return null;
  }

  if (
    typeof navigator !== "undefined" &&
    navigator.userActivation?.hasBeenActive === false
  ) {
    return null;
  }

  const ContextConstructor =
    window.AudioContext ?? (window as WebKitWindow).webkitAudioContext;
  if (!ContextConstructor) {
    return null;
  }

  try {
    return new ContextConstructor();
  } catch {
    return null;
  }
}

export async function ensureAudioContextRunning(
  context: AudioContext,
): Promise<boolean> {
  if (isAudioContextRunning(context)) {
    return true;
  }

  try {
    await context.resume();
    return isAudioContextRunning(context);
  } catch {
    return false;
  }
}

function isAudioContextRunning(context: AudioContext): boolean {
  return context.state === "running";
}
