import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import { config, setup, mountApp } from "./app.js";

describe("chargement", () => {
  it("affiche les événements du premier jeu", async () => {
    setup();
    const w = await mountApp();
    expect(invoke).toHaveBeenCalledWith("get_config", undefined);
    const names = w.findAll(".event-name").map((e) => e.text());
    expect(names).toEqual(["Mort", "Boss vaincu +2 jokers"]);
    expect(w.text()).toContain("Rien en attente");
  });

  it("reprend le dernier jeu choisi", async () => {
    setup({ get_last_game: "ds" });
    const w = await mountApp();
    expect(w.find("select").element.value).toBe("ds");
    expect(w.findAll(".event-name").map((e) => e.text())).toEqual(["Feu de camp"]);
  });

  it("revient au premier jeu si le jeu mémorisé n'existe plus", async () => {
    setup({ get_last_game: "inconnu" });
    await mountApp();
    expect(invoke).toHaveBeenLastCalledWith("set_last_game", { gameId: "er" });
  });

  it("enregistre le jeu choisi", async () => {
    setup();
    const w = await mountApp();
    await w.find("select").setValue("ds");
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith("set_last_game", { gameId: "ds" });
  });

  it("affiche l'erreur du backend", async () => {
    setup({ get_config: new Error("config illisible") });
    const w = await mountApp();
    expect(w.find(".banner.error").text()).toContain("config illisible");
  });

  it("signale une config invalide hors de l'onglet Configuration", async () => {
    setup({ get_config: { config, error: "JSON invalide" } });
    const w = await mountApp();
    expect(w.find(".banner.error").text()).toContain("JSON invalide");
    await w.find(".banner.error button").trigger("click");
    expect(w.find(".cfg-events").exists()).toBe(true);
  });
});
