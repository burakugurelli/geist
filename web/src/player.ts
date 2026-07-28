import { ensureAudioContextRunning } from "./audio-context.js";
import { isCueName, type CueName } from "./cue.js";
import type { PcmRenderer } from "./renderer.js";

type PlayerDependencies = {
  renderer: PcmRenderer;
  createContext: () => AudioContext | null;
};

export class Player {
  private context: AudioContext | null = null;
  private readonly buffers = new Map<string, Promise<AudioBuffer>>();

  constructor(private readonly dependencies: PlayerDependencies) {}

  play(value: unknown): void {
    if (!isCueName(value)) {
      return;
    }

    const context = this.getContext();
    if (!context) {
      return;
    }

    void this.playWhenReady(context, value).catch(() => undefined);
  }

  private getContext(): AudioContext | null {
    if (this.context?.state === "closed") {
      this.context = null;
    }

    this.context ??= this.dependencies.createContext();
    return this.context;
  }

  private async playWhenReady(
    context: AudioContext,
    cue: CueName,
  ): Promise<void> {
    if (!(await ensureAudioContextRunning(context))) {
      return;
    }

    const buffer = await this.getBuffer(context, cue);
    const source = context.createBufferSource();
    source.buffer = buffer;
    source.connect(context.destination);
    source.start();
  }

  private async getBuffer(
    context: AudioContext,
    cue: CueName,
  ): Promise<AudioBuffer> {
    const key = `${cue}:${context.sampleRate}`;
    const cached = this.buffers.get(key);
    if (cached) {
      return cached;
    }

    const pending = this.createBuffer(context, cue);
    this.buffers.set(key, pending);

    try {
      return await pending;
    } catch (error: unknown) {
      if (this.buffers.get(key) === pending) {
        this.buffers.delete(key);
      }
      throw error;
    }
  }

  private async createBuffer(
    context: AudioContext,
    cue: CueName,
  ): Promise<AudioBuffer> {
    const samples = await this.dependencies.renderer.render(
      cue,
      context.sampleRate,
    );
    const buffer = context.createBuffer(1, samples.length, context.sampleRate);
    buffer.getChannelData(0).set(samples);
    return buffer;
  }
}
