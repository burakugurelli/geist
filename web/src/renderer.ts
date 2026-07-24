import initializeGeneratedWasm, {
  render as renderWasm,
} from "../generated/geist_wasm.js";

import type { CueName } from "./cue.js";

export interface PcmRenderer {
  render(cue: CueName, sampleRate: number): Promise<Float32Array>;
}

type InitializeWasm = () => Promise<unknown>;
type RenderWasm = (cue: string, sampleRate: number) => Float32Array;

export class WasmRenderer implements PcmRenderer {
  private initialization: Promise<void> | null = null;

  constructor(
    private readonly initializeModule: InitializeWasm = () =>
      initializeGeneratedWasm(),
    private readonly renderSamples: RenderWasm = renderWasm,
  ) {}

  async render(cue: CueName, sampleRate: number): Promise<Float32Array> {
    await this.initialize();
    return this.renderSamples(cue, sampleRate);
  }

  private initialize(): Promise<void> {
    if (!this.initialization) {
      this.initialization = this.initializeModule()
        .then(() => undefined)
        .catch((error: unknown) => {
          // A transient module-loading failure must not poison later play calls.
          this.initialization = null;
          throw error;
        });
    }

    return this.initialization;
  }
}
