import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import App from "./App.vue";
import { slogans } from "./slogans.js";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const config = {
  exercises: [{ id: "squat", name: "Squats", unit: "reps", min: 10, max: 15 }],
  games: [
    {
      id: "er",
      name: "Elden Ring",
      events: [
        { id: "mort", name: "Mort", draws: 1, maxPerSession: 15 },
        { id: "boss", name: "Boss vaincu", draws: 1, jokers: 2 },
      ],
    },
    { id: "ds", name: "Dark Souls", events: [{ id: "feu", name: "Feu de camp", draws: 1 }] },
  ],
};

function emptySession(extra = {}) {
  return { pending: [], sets: 0, totals: [], log: [], triggers: [], jokers: 0, jokersEarned: 0, skipped: [], ...extra };
}

const draw = { id: 7, eventName: "Mort", gameName: "Elden Ring", jokers: 0, items: [{ exerciseId: "squat", name: "Squats", amount: 10, unit: "reps" }] };

// Réponses du backend simulé, modifiables par test.
let responses;
// Composants montés, démontés après chaque test pour retirer leur écouteur clavier.
let mounted = [];

function setup(over = {}) {
  responses = {
    get_config: { config },
    get_session: emptySession(),
    get_history: [],
    get_last_game: "",
    set_last_game: undefined,
    ...over,
  };
  invoke.mockImplementation(async (cmd, args) => {
    const r = responses[cmd];
    if (r instanceof Error) throw r;
    return typeof r === "function" ? r(args) : r;
  });
}

async function mountApp() {
  const wrapper = mount(App, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

function press(code, init = {}) {
  window.dispatchEvent(new KeyboardEvent("keydown", { code, bubbles: true, cancelable: true, ...init }));
  return flushPromises();
}

beforeEach(() => {
  invoke.mockReset();
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  vi.useRealTimers();
});

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

describe("bilan", () => {
  it("charge l'historique des 7 derniers jours", async () => {
    setup({
      get_session: emptySession({
        startedAt: 0,
        sets: 1,
        totals: [{ exercise: "Squats", unit: "reps", sets: 1, amount: 10 }],
        log: [{ at: 0, amount: 10, unit: "reps", exercise: "Squats", event: "Mort" }],
      }),
      get_history: [
        { date: "2026-10-10", sets: 0, sessions: [] },
        {
          date: "2026-10-11",
          sets: 1,
          sessions: [
            { start: 0, duration: 3900, game: "Elden Ring", sets: 1, totals: [{ exercise: "Squats", unit: "reps", amount: 10 }] },
          ],
        },
      ],
    });
    const w = await mountApp();
    expect(w.find(".tabs .badge").text()).toBe("1");
    await w.findAll(".tabs button")[1].trigger("click");
    await flushPromises();
    expect(invoke).toHaveBeenCalledWith("get_history", { days: 7 });
    expect(w.find(".totals").text()).toContain("10 reps");
    const days = w.findAll(".days li");
    expect(days[0].text()).toContain("repos");
    expect(days[1].text()).toContain("Squats 10 reps");
    expect(days[1].text()).toContain("Elden Ring");
    expect(days[1].text()).toContain("1 h 05");
  });
});

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

describe("slogan", () => {
  it("affiche un slogan de la liste", async () => {
    setup();
    const w = await mountApp();
    expect(slogans).toContain(w.find(".splash").text());
  });
});
