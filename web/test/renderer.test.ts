import { describe, expect, it, vi } from "vitest";

import { WasmRenderer } from "../src/renderer.js";

describe("WasmRenderer", () => {
  it("shares one initialization across concurrent renders", async () => {
    const initialize = vi.fn(async () => undefined);
    const renderSamples = vi.fn(() => new Float32Array([0, 0.5, 0]));
    const renderer = new WasmRenderer(initialize, renderSamples);

    const [press, success] = await Promise.all([
      renderer.render("press", 48_000),
      renderer.render("success", 48_000),
    ]);

    expect(initialize).toHaveBeenCalledTimes(1);
    expect(renderSamples).toHaveBeenCalledTimes(2);
    expect(press).toEqual(new Float32Array([0, 0.5, 0]));
    expect(success).toEqual(new Float32Array([0, 0.5, 0]));
  });

  it("retries initialization after a failure", async () => {
    const initialize = vi
      .fn<() => Promise<unknown>>()
      .mockRejectedValueOnce(new Error("temporary failure"))
      .mockResolvedValue(undefined);
    const renderSamples = vi.fn(() => new Float32Array([0.25]));
    const renderer = new WasmRenderer(initialize, renderSamples);

    await expect(renderer.render("error", 48_000)).rejects.toThrow(
      "temporary failure",
    );
    await expect(renderer.render("error", 48_000)).resolves.toEqual(
      new Float32Array([0.25]),
    );
    expect(initialize).toHaveBeenCalledTimes(2);
  });
});
