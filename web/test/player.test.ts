import { describe, expect, it, vi } from "vitest";

import type { PcmRenderer } from "../src/renderer.js";
import { Player } from "../src/player.js";

class FakeAudioContext {
  state: AudioContextState = "running";
  readonly destination = {} as AudioDestinationNode;
  readonly resume = vi.fn(async () => {
    this.state = "running";
  });
  readonly copiedSamples: Float32Array[] = [];
  sourceStarts = 0;
  buffersCreated = 0;
  sourcesCreated = 0;

  constructor(readonly sampleRate: number) {}

  createBuffer(
    _channels: number,
    length: number,
    _sampleRate: number,
  ): AudioBuffer {
    this.buffersCreated += 1;
    return {
      copyToChannel: (samples: Float32Array) => {
        expect(samples).toHaveLength(length);
        this.copiedSamples.push(samples);
      },
    } as unknown as AudioBuffer;
  }

  createBufferSource(): AudioBufferSourceNode {
    this.sourcesCreated += 1;
    const context = this;
    return {
      buffer: null,
      connect: vi.fn(),
      start() {
        context.sourceStarts += 1;
      },
    } as unknown as AudioBufferSourceNode;
  }
}

function createRenderer() {
  return {
    render: vi.fn(async () => new Float32Array([0, 0.25, 0])),
  } satisfies PcmRenderer;
}

describe("Player", () => {
  it("ignores invalid runtime cue values before creating audio", () => {
    const renderer = createRenderer();
    const context = new FakeAudioContext(48_000);
    const createContext = vi.fn(() => context as unknown as AudioContext);
    const player = new Player({ renderer, createContext });

    player.play("chime");

    expect(createContext).not.toHaveBeenCalled();
    expect(renderer.render).not.toHaveBeenCalled();
  });

  it("reuses one context and one rendered buffer", async () => {
    const renderer = createRenderer();
    const context = new FakeAudioContext(48_000);
    const createContext = vi.fn(() => context as unknown as AudioContext);
    const player = new Player({ renderer, createContext });

    player.play("press");
    await vi.waitFor(() => expect(context.sourceStarts).toBe(1));
    player.play("press");
    await vi.waitFor(() => expect(context.sourceStarts).toBe(2));

    expect(createContext).toHaveBeenCalledTimes(1);
    expect(renderer.render).toHaveBeenCalledTimes(1);
    expect(context.buffersCreated).toBe(1);
    expect(context.sourcesCreated).toBe(2);
  });

  it("shares an in-flight render between concurrent calls", async () => {
    let finishRender: ((samples: Float32Array) => void) | undefined;
    const renderer: PcmRenderer = {
      render: vi.fn(
        () =>
          new Promise<Float32Array>((resolve) => {
            finishRender = resolve;
          }),
      ),
    };
    const context = new FakeAudioContext(48_000);
    const player = new Player({
      renderer,
      createContext: () => context as unknown as AudioContext,
    });

    player.play("success");
    player.play("success");
    await vi.waitFor(() => expect(renderer.render).toHaveBeenCalledTimes(1));
    finishRender?.(new Float32Array([0.1]));
    await vi.waitFor(() => expect(context.sourceStarts).toBe(2));
  });

  it("uses a new cache entry after a closed context changes sample rate", async () => {
    const renderer = createRenderer();
    const first = new FakeAudioContext(44_100);
    const second = new FakeAudioContext(48_000);
    const createContext = vi
      .fn<() => AudioContext | null>()
      .mockReturnValueOnce(first as unknown as AudioContext)
      .mockReturnValueOnce(second as unknown as AudioContext);
    const player = new Player({ renderer, createContext });

    player.play("error");
    await vi.waitFor(() => expect(first.sourceStarts).toBe(1));
    first.state = "closed";
    player.play("error");
    await vi.waitFor(() => expect(second.sourceStarts).toBe(1));

    expect(renderer.render).toHaveBeenNthCalledWith(1, "error", 44_100);
    expect(renderer.render).toHaveBeenNthCalledWith(2, "error", 48_000);
  });

  it("retries rendering after a failure", async () => {
    const renderer = {
      render: vi
        .fn<
          (
            cue: "press" | "success" | "error",
            rate: number,
          ) => Promise<Float32Array>
        >()
        .mockRejectedValueOnce(new Error("temporary"))
        .mockResolvedValue(new Float32Array([0.2])),
    } satisfies PcmRenderer;
    const context = new FakeAudioContext(48_000);
    const player = new Player({
      renderer,
      createContext: () => context as unknown as AudioContext,
    });

    player.play("press");
    await vi.waitFor(() => expect(renderer.render).toHaveBeenCalledTimes(1));
    await new Promise((resolve) => setTimeout(resolve, 0));
    player.play("press");
    await vi.waitFor(() => expect(context.sourceStarts).toBe(1));

    expect(renderer.render).toHaveBeenCalledTimes(2);
  });

  it("does no rendering when audio cannot resume", async () => {
    const renderer = createRenderer();
    const context = new FakeAudioContext(48_000);
    context.state = "suspended";
    context.resume.mockRejectedValue(new Error("blocked"));
    const player = new Player({
      renderer,
      createContext: () => context as unknown as AudioContext,
    });

    player.play("success");
    await vi.waitFor(() => expect(context.resume).toHaveBeenCalledTimes(1));

    expect(renderer.render).not.toHaveBeenCalled();
    expect(context.sourceStarts).toBe(0);
  });
});
