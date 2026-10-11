import { describe, expect, it } from "vitest";
import { byName, newId, normalized } from "../configDraft.js";

describe("newId", () => {
  it("dérive l'id du nom sans accents et évite les doublons", () => {
    expect(newId("Événement Spécial", [])).toBe("evenement-special");
    expect(newId("Mort", [{ id: "mort" }, { id: "mort-2" }])).toBe("mort-3");
    expect(newId("!!", [])).toBe("id");
  });
});

describe("byName", () => {
  it("trie par nom sans tenir compte de la casse ni des accents, sans toucher l'original", () => {
    const games = [{ name: "sekiro" }, { name: "Élden Ring" }, { name: "Bloodborne" }];
    expect(byName(games).map((g) => g.name)).toEqual(["Bloodborne", "Élden Ring", "sekiro"]);
    expect(games[0].name).toBe("sekiro");
  });
});

describe("normalized", () => {
  it("ramène les champs numériques vidés à 0 sans toucher l'original", () => {
    const cfg = { exercises: [{ min: "", max: 5 }], games: [{ events: [{ draws: "", jokers: undefined }] }] };
    expect(normalized(cfg)).toEqual({ exercises: [{ min: 0, max: 5 }], games: [{ events: [{ draws: 0, jokers: 0 }] }] });
    expect(cfg.exercises[0].min).toBe("");
  });
});
