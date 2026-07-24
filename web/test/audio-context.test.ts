import { afterEach, describe, expect, it, vi } from "vitest";

import {
  createAudioContext,
  ensureAudioContextRunning,
} from "../src/audio-context.js";

afterEach(() => {
  vi.unstubAllGlobals();
});

function stubActivatedBrowser(
  constructors: {
    AudioContext?: typeof AudioContext;
    webkitAudioContext?: typeof AudioContext;
  },
): void {
  vi.stubGlobal("navigator", {
    userActivation: { hasBeenActive: true },
  });
  vi.stubGlobal("window", constructors);
}

function createContextConstructor(context: AudioContext) {
  return vi.fn(function AudioContextConstructor() {
    return context;
  });
}

function createResumableContext(
  initialState: AudioContextState,
  resumedState = initialState,
) {
  const context = {
    state: initialState,
    resume: vi.fn(async () => {
      context.state = resumedState;
    }),
  };

  return context;
}

describe("createAudioContext", () => {
  it("is a no-op during server-side rendering", () => {
    expect(createAudioContext()).toBeNull();
  });

  it("constructs an audio context after user activation", () => {
    const context = { state: "suspended" } as AudioContext;
    const constructor = createContextConstructor(context);
    stubActivatedBrowser({
      AudioContext: constructor as unknown as typeof AudioContext,
    });

    expect(createAudioContext()).toBe(context);
    expect(constructor).toHaveBeenCalledTimes(1);
  });

  it("uses the WebKit constructor when AudioContext is unavailable", () => {
    const context = { state: "suspended" } as AudioContext;
    const constructor = createContextConstructor(context);
    stubActivatedBrowser({
      webkitAudioContext: constructor as unknown as typeof AudioContext,
    });

    expect(createAudioContext()).toBe(context);
    expect(constructor).toHaveBeenCalledTimes(1);
  });

  it("waits for user activation", () => {
    const constructor = vi.fn();
    vi.stubGlobal("navigator", {
      userActivation: { hasBeenActive: false },
    });
    vi.stubGlobal("window", { AudioContext: constructor });

    expect(createAudioContext()).toBeNull();
    expect(constructor).not.toHaveBeenCalled();
  });

  it("returns null when construction is blocked", () => {
    class BlockedContext {
      constructor() {
        throw new Error("blocked");
      }
    }

    vi.stubGlobal("navigator", {
      userActivation: { hasBeenActive: true },
    });
    vi.stubGlobal("window", { AudioContext: BlockedContext });

    expect(createAudioContext()).toBeNull();
  });
});

describe("ensureAudioContextRunning", () => {
  it("does not resume an already running context", async () => {
    const resume = vi.fn();
    const context = { state: "running", resume } as unknown as AudioContext;

    await expect(ensureAudioContextRunning(context)).resolves.toBe(true);
    expect(resume).not.toHaveBeenCalled();
  });

  it("resumes a suspended context", async () => {
    const context = createResumableContext("suspended", "running");

    await expect(
      ensureAudioContextRunning(context as unknown as AudioContext),
    ).resolves.toBe(true);
    expect(context.resume).toHaveBeenCalledTimes(1);
  });

  it("returns false when resume does not start the context", async () => {
    const context = createResumableContext("suspended");

    await expect(
      ensureAudioContextRunning(context as unknown as AudioContext),
    ).resolves.toBe(false);
    expect(context.resume).toHaveBeenCalledTimes(1);
  });

  it("attempts to resume a closed context and returns false", async () => {
    const context = createResumableContext("closed");

    await expect(
      ensureAudioContextRunning(context as unknown as AudioContext),
    ).resolves.toBe(false);
    expect(context.resume).toHaveBeenCalledTimes(1);
  });

  it("resumes an interrupted Safari context", async () => {
    const context = createResumableContext("interrupted", "running");

    await expect(
      ensureAudioContextRunning(context as unknown as AudioContext),
    ).resolves.toBe(true);
    expect(context.resume).toHaveBeenCalledTimes(1);
  });

  it("returns false when resume is rejected", async () => {
    const context = {
      state: "suspended",
      resume: vi.fn().mockRejectedValue(new Error("blocked")),
    } as unknown as AudioContext;

    await expect(ensureAudioContextRunning(context)).resolves.toBe(false);
  });
});
