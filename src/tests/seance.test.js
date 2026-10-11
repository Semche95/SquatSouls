import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import { emptySession, draw, responses, setup, mountApp } from "./app.js";

describe("séance", () => {
  it("tire une série au clic sur un événement", async () => {
    setup({ draw: { id: 7, capped: false } });
    const w = await mountApp();
    responses.get_session = emptySession({ pending: [draw], triggers: [{ gameId: "er", eventId: "mort", count: 1 }] });
    await w.find(".event").trigger("click");
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith("draw", { gameId: "er", eventId: "mort" });
    expect(w.find(".card.current").text()).toContain("Squats");
    expect(w.find(".event .count").text()).toBe("1 / 15");
  });

  it("prévient quand la limite par séance est atteinte", async () => {
    setup({ draw: { capped: true, eventName: "Mort", maxPerSession: 15 } });
    const w = await mountApp();
    await w.find(".event").trigger("click");
    await flushPromises();
    expect(w.find(".banner.info").text()).toBe("Mort : limite de 15 atteinte pour la séance, pas de série.");
  });

  it("valide, annule ou saute une série", async () => {
    setup({
      get_session: emptySession({ pending: [draw], jokers: 1 }),
      complete: emptySession(),
      dismiss: emptySession(),
      use_joker: emptySession(),
    });
    const w = await mountApp();
    const buttons = w.findAll(".card-actions button").map((b) => b.text());
    expect(buttons).toEqual(["Fait Entrée", "Joker J", "Annuler Retour arr."]);
    await w.find(".card-actions .primary").trigger("click");
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith("complete", { drawId: 7 });
    expect(w.find(".card").exists()).toBe(false);
  });

  it("recharge la séance si une action échoue", async () => {
    setup({ get_session: emptySession({ pending: [draw] }), complete: new Error("série inconnue") });
    const w = await mountApp();
    responses.get_session = emptySession();
    await w.find(".card-actions .primary").trigger("click");
    await flushPromises();
    expect(w.find(".banner.error").text()).toContain("série inconnue");
    expect(w.find(".card").exists()).toBe(false);
  });

  it("verrouille le jeu pendant la séance", async () => {
    setup({ get_session: emptySession({ gameId: "ds" }) });
    const w = await mountApp();
    const select = w.find("select");
    expect(select.element.disabled).toBe(true);
    expect(select.element.value).toBe("ds");
  });

  it("ne propose de terminer la séance qu'avec des séries faites et aucune en attente", async () => {
    setup({ get_session: emptySession() });
    let w = await mountApp();
    expect(w.find(".session-actions .primary").element.disabled).toBe(true);
    setup({ get_session: emptySession({ sets: 1, pending: [draw] }) });
    w = await mountApp();
    expect(w.find(".session-actions .primary").element.disabled).toBe(true);
  });

  it("demande confirmation avant de terminer la séance, puis enchaîne sur une nouvelle", async () => {
    setup({ get_session: emptySession({ sets: 1 }), new_session: emptySession() });
    const w = await mountApp();
    await w.find(".session-actions .primary").trigger("click");
    expect(invoke).not.toHaveBeenCalledWith("new_session", undefined);
    expect(w.text()).toContain("Terminer la séance ?");
    await w.find(".confirm .ghost").trigger("click");
    expect(w.find(".confirm").exists()).toBe(false);
    await w.find(".session-actions .primary").trigger("click");
    await w.find(".confirm .primary").trigger("click");
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith("new_session", undefined);
    expect(w.find(".confirm").exists()).toBe(false);
    expect(w.find(".event").element.disabled).toBe(false);
  });

  it("désactive l'annulation tant que la séance est vide", async () => {
    setup();
    const w = await mountApp();
    expect(w.find(".session-actions .danger").element.disabled).toBe(true);
  });

  it("demande confirmation avant d'annuler la séance", async () => {
    setup({ get_session: emptySession({ gameId: "er" }), cancel_session: emptySession() });
    const w = await mountApp();
    await w.find(".session-actions .danger").trigger("click");
    expect(w.text()).toContain("Annuler la séance ?");
    await w.find(".confirm .ghost").trigger("click");
    expect(w.find(".confirm").exists()).toBe(false);
    expect(invoke).not.toHaveBeenCalledWith("cancel_session", undefined);
    await w.find(".session-actions .danger").trigger("click");
    await w.find(".confirm .danger").trigger("click");
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith("cancel_session", undefined);
    expect(invoke).not.toHaveBeenCalledWith("new_session", undefined);
  });
});
