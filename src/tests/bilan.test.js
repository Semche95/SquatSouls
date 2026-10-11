import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { invoke } from "@tauri-apps/api/core";
import { emptySession, setup, mountApp } from "./app.js";

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
