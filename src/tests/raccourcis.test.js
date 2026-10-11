import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import { emptySession, draw, responses, setup, mountApp, press } from "./app.js";

describe("raccourcis clavier", () => {
  it("déclenche l'événement correspondant au chiffre", async () => {
    setup({ draw: { capped: false } });
    await mountApp();
    await press("Digit2");
    expect(invoke).toHaveBeenCalledWith("draw", { gameId: "er", eventId: "boss" });
    await press("Numpad1");
    expect(invoke).toHaveBeenCalledWith("draw", { gameId: "er", eventId: "mort" });
  });

  it("ignore un chiffre sans événement et les répétitions", async () => {
    setup({ draw: { capped: false } });
    await mountApp();
    await press("Digit9");
    await press("Digit1", { repeat: true });
    expect(invoke).not.toHaveBeenCalledWith("draw", expect.anything());
  });

  it("agit sur la plus ancienne série", async () => {
    const other = { ...draw, id: 8 };
    setup({ get_session: emptySession({ pending: [draw, other], jokers: 1 }), complete: () => responses.get_session, dismiss: () => responses.get_session, use_joker: () => responses.get_session });
    await mountApp();
    await press("Enter");
    expect(invoke).toHaveBeenCalledWith("complete", { drawId: 7 });
    await press("Backspace");
    expect(invoke).toHaveBeenCalledWith("dismiss", { drawId: 7 });
    await press("KeyJ");
    expect(invoke).toHaveBeenCalledWith("use_joker", { drawId: 7 });
  });

  it("n'utilise pas de joker sans joker disponible", async () => {
    setup({ get_session: emptySession({ pending: [draw] }) });
    await mountApp();
    await press("KeyJ");
    expect(invoke).not.toHaveBeenCalledWith("use_joker", expect.anything());
  });

  it("est inactif hors de l'onglet Jouer et après la fin de séance", async () => {
    setup({ draw: { capped: false } });
    const w = await mountApp();
    await w.findAll(".tabs button")[2].trigger("click");
    await press("Digit1");
    expect(invoke).not.toHaveBeenCalledWith("draw", expect.anything());

  });

  it("ignore les touches tapées dans un champ", async () => {
    setup({ draw: { capped: false } });
    const w = await mountApp();
    w.find("select").element.dispatchEvent(new KeyboardEvent("keydown", { code: "Digit1", bubbles: true }));
    await flushPromises();
    expect(invoke).not.toHaveBeenCalledWith("draw", expect.anything());
  });
});
