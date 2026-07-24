import { createAudioContext } from "./audio-context.js";
import type { CueName } from "./cue.js";
import { Player } from "./player.js";
import { WasmRenderer } from "./renderer.js";

const player = new Player({
  renderer: new WasmRenderer(),
  createContext: createAudioContext,
});

export type { CueName };

/** Plays a semantic interface cue when browser audio is available. */
export function play(cue: CueName): void {
  player.play(cue);
}
