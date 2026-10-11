import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import { config, responses, setup, mountApp } from "./app.js";

describe("configuration", () => {
  async function openConfig() {
    const w = await mountApp();
    await w.findAll(".tabs button")[2].trigger("click");
    return w;
  }

  it("affiche dans la liste le jeu en cours d'édition", async () => {
    setup();
    const w = await openConfig();
    expect(w.find(".cfg-game-list .active").text()).toBe("Elden Ring");
  });

  it("édite la configuration avec des champs et l'applique dès l'enregistrement", async () => {
    vi.useFakeTimers();
    setup();
    const w = await openConfig();
    const save = () => w.find(".cfg-actions .primary");
    expect(save().exists()).toBe(false);
    await w.find('input[aria-label="Nom de l\'événement"]').setValue("Mort brutale");
    responses.save_config = (args) => ({ config: args.config });
    await save().trigger("click");
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === "save_config")[1].config;
    expect(sent.games[0].events[0]).toMatchObject({ id: "mort", name: "Mort brutale" });
    expect(w.find(".banner.ok").text()).toBe("Configuration enregistrée.");
    await vi.runAllTimersAsync();
    expect(w.find(".banner.ok").exists()).toBe(false);
    await w.findAll(".tabs button")[0].trigger("click");
    expect(w.find(".event").text()).toContain("Mort brutale");
  });

  it("ajoute un exercice avec un id dérivé du nom", async () => {
    setup();
    const w = await openConfig();
    await w.findAll(".subtabs button")[1].trigger("click");
    expect(w.find(".cfg-game-list").exists()).toBe(false);
    const add = w.find(".cfg-exercises > button");
    await add.trigger("click");
    await add.trigger("click");
    responses.save_config = (args) => ({ config: args.config });
    await w.find(".cfg-actions .primary").trigger("click");
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === "save_config")[1].config;
    expect(sent.exercises.map((e) => e.id)).toEqual(["squat", "nouvel-exercice", "nouvel-exercice-2"]);
  });

  it("crée un jeu vide ou avec les événements copiés d'un autre jeu", async () => {
    setup();
    const w = await openConfig();
    async function create(name, source) {
      await w.find(".group-head button").trigger("click");
      expect(w.find("form.cfg-game .primary").element.disabled).toBe(true);
      await w.find("input[aria-label='Nom du nouveau jeu']").setValue(name);
      await w.find("select[aria-label='Événements du nouveau jeu']").setValue(source);
      await w.find("form.cfg-game").trigger("submit");
    }
    await create("Sekiro", "ds");
    expect(w.find(".cfg-game-list .active").text()).toBe("Sekiro");
    await create("Lies of P", "");
    responses.save_config = (args) => ({ config: args.config });
    await w.find(".cfg-actions .primary").trigger("click");
    await flushPromises();
    const [, , sekiro, lies] = invoke.mock.calls.find(([cmd]) => cmd === "save_config")[1].config.games;
    expect(sekiro).toMatchObject({ id: "sekiro", events: config.games[1].events });
    expect(lies).toMatchObject({ id: "lies-of-p", events: [{ name: "Nouvel événement", draws: 1 }] });
  });

  it("règle l'intervalle d'un exercice et garde au moins un exercice", async () => {
    setup();
    const w = await openConfig();
    await w.findAll(".subtabs button")[1].trigger("click");
    expect(w.find(".cfg-exercises .icon").element.disabled).toBe(true);
    await w.find("input[aria-label='Maximum']").setValue("");
    responses.save_config = (args) => ({ config: args.config });
    await w.find(".cfg-actions .primary").trigger("click");
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === "save_config")[1].config;
    expect(sent.exercises[0]).toMatchObject({ min: 10, max: 0 });
  });

  it("vide la limite par séance pour la retirer", async () => {
    setup();
    const w = await openConfig();
    await w.find(".cfg-params input[placeholder='aucune']").setValue("");
    responses.save_config = (args) => ({ config: args.config });
    await w.find(".cfg-actions .primary").trigger("click");
    await flushPromises();
    const sent = invoke.mock.calls.find(([cmd]) => cmd === "save_config")[1].config;
    expect(sent.games[0].events[0]).not.toHaveProperty("maxPerSession");
  });

  it("réinitialise les modifications non enregistrées", async () => {
    setup();
    const w = await openConfig();
    const name = () => w.find("input[aria-label='Nom du jeu']");
    await name().setValue("Autre");
    await w.find(".cfg-actions .ghost").trigger("click");
    expect(name().element.value).toBe("Elden Ring");
    expect(w.find(".cfg-actions").exists()).toBe(false);
  });

  it("affiche l'erreur d'enregistrement", async () => {
    setup({ save_config: new Error("champ manquant") });
    const w = await openConfig();
    await w.find("input[aria-label='Nom du jeu']").setValue("Autre");
    await w.find(".cfg-actions .primary").trigger("click");
    await flushPromises();
    expect(w.find(".banner.error").text()).toContain("champ manquant");
  });

});
