import { play, type CueName } from "../../web/src/index.js";

type SoundControl = {
  readonly buttonId: string;
  readonly cue: CueName;
};

const controls: ReadonlyArray<SoundControl> = [
  { buttonId: "play-press", cue: "press" },
  { buttonId: "play-success", cue: "success" },
  { buttonId: "play-error", cue: "error" },
];

const status = document.querySelector<HTMLElement>("#playback-status");
if (!status) {
  throw new Error("The demo playback status element is missing.");
}

for (const { buttonId, cue } of controls) {
  const button = document.querySelector<HTMLButtonElement>(`#${buttonId}`);
  if (!button) {
    throw new Error(`The demo button #${buttonId} is missing.`);
  }

  button.addEventListener("click", () => {
    try {
      play(cue);
      status.dataset.state = "requested";
      status.textContent = `Requested the ${cue} cue.`;
    } catch (error: unknown) {
      status.dataset.state = "error";
      status.textContent =
        error instanceof Error
          ? `Could not request the ${cue} cue: ${error.message}`
          : `Could not request the ${cue} cue.`;
    }
  });
}
