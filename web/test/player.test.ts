import { describe, expect, it, vi } from "vitest";

import type { PcmRenderer } from "../src/renderer.js";
import { Player } from "../src/player.js";

class FakeAudioBuffer {
  readonly channelData: Float32Array<ArrayBuffer>;
  readonly getChannelData = vi.fn(
    (_channelNumber: number): Float32Array<ArrayBuffer> => this.channelData,
  );

  constructor(length: number) {
    this.channelData = new Float32Array(length);
  }
}

class FakeAudioBufferSource {
  buffer: AudioBuffer | null = null;
  readonly connect = vi.fn<(destination: AudioNode) => AudioNode>();
  readonly start = vi.fn<() => void>();
}

class FakeAudioContext {
  state: AudioContextState = "running";
  readonly destination = {} as AudioDestinationNode;
  readonly resume = vi.fn(async () => {
    this.state = "running";
  });
  readonly buffers: FakeAudioBuffer[] = [];
  readonly sources: FakeAudioBufferSource[] = [];
  readonly createBuffer = vi.fn(
    (
      _numberOfChannels: number,
      length: number,
      _sampleRate: number,
    ): AudioBuffer => {
      const buffer = new FakeAudioBuffer(length);
      this.buffers.push(buffer);
      return buffer as unknown as AudioBuffer;
    },
  );
  readonly createBufferSource = vi.fn((): AudioBufferSourceNode => {
    const source = new FakeAudioBufferSource();
    this.sources.push(source);
    return source as unknown as AudioBufferSourceNode;
  });

  constructor(readonly sampleRate: number) {}

  get sourceStarts(): number {
    return this.sources.reduce(
      (count, source) => count + source.start.mock.calls.length,
      0,
    );
  }

  get buffersCreated(): number {
    return this.buffers.length;
  }

  get sourcesCreated(): number {
    return this.sources.length;
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

  it("routes rendered PCM through a mono buffer to the destination", async () => {
    const samples = new Float32Array([0, 0.25, -0.125]);
    const renderer = {
      render: vi.fn(async () => samples),
    } satisfies PcmRenderer;
    const context = new FakeAudioContext(48_000);
    const player = new Player({
      renderer,
      createContext: () => context as unknown as AudioContext,
    });

    player.play("press");
    await vi.waitFor(() => expect(context.sourceStarts).toBe(1));

    expect(context.createBuffer).toHaveBeenCalledWith(
      1,
      samples.length,
      context.sampleRate,
    );
    expect(context.createBuffer).toHaveBeenCalledOnce();
    expect(context.buffers).toHaveLength(1);
    expect(context.sources).toHaveLength(1);

    const buffer = context.buffers[0]!;
    const source = context.sources[0]!;
    expect(buffer.getChannelData).toHaveBeenCalledWith(0);
    expect(buffer.getChannelData).toHaveBeenCalledOnce();
    expect(buffer.channelData).toEqual(samples);
    expect(source.buffer).toBe(buffer);
    expect(source.connect).toHaveBeenCalledWith(context.destination);
    expect(source.connect).toHaveBeenCalledOnce();
    expect(source.start).toHaveBeenCalledOnce();
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
