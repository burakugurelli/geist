import { describe, expect, it } from "vitest";

describe("public web entry point", () => {
  it("is safe to import without browser globals", async () => {
    const publicApi = await import("../src/index.js");

    expect(Object.keys(publicApi)).toEqual(["play"]);
    expect(publicApi.play).toBeTypeOf("function");
  });
});
