import { describe, expect, it } from "vitest";
import { slogans } from "../slogans.js";
import { setup, mountApp } from "./app.js";

describe("slogan", () => {
  it("affiche un slogan de la liste", async () => {
    setup();
    const w = await mountApp();
    expect(slogans).toContain(w.find(".splash").text());
  });
});
