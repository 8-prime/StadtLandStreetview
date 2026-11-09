import { describe, expect, it } from "vitest";
import {
  applyAction,
  createRoom,
  currentFinding,
  demoView,
  randomStarts,
  reconcile,
  simulatePeers,
  snapshot,
  tick,
  type Room,
} from "./engine";

function room() {
  const r = createRoom("TEST12", "Alex", "alex", 60, true);
  r.players.forEach((p) => (p.bot = false));
  return r;
}
function readyAll(r: Room) {
  const stepId = r.stepId;
  for (const p of r.players.filter((p) => !p.left))
    applyAction(r, p.id, { type: "ready", stepId, ready: true }, 1000);
}
function searching() {
  const r = room();
  readyAll(r);
  readyAll(r);
  for (const player of r.players.slice(1)) {
    for (const category of r.categories) {
      applyAction(
        r,
        player.id,
        {
          type: "save-finding",
          categoryId: category.id,
          view: demoView(player.color),
        },
        2000,
      );
    }
  }
  return r;
}

describe("peer progression and findings", () => {
  it("allows undo, waits for all peers, resets readiness, and rejects stale acknowledgements", () => {
    const r = room();
    const oldStep = r.stepId;
    applyAction(r, "alex", { type: "ready", stepId: oldStep, ready: true });
    expect(r.phase).toBe("lobby");
    applyAction(r, "alex", { type: "ready", stepId: oldStep, ready: false });
    expect(r.players[0].ready).toBe(false);
    readyAll(r);
    expect(r.phase).toBe("categories");
    expect(r.players.every((p) => !p.ready)).toBe(true);
    expect(() =>
      applyAction(r, "alex", { type: "ready", stepId: oldStep, ready: true }),
    ).toThrow("moved on");
  });

  it("lets any lobby member change the room rules and resets readiness on a change", () => {
    const r = room();
    applyAction(r, "alex", { type: "ready", stepId: r.stepId, ready: true });
    const step = r.stepId;
    const mika = r.players[1].id;
    applyAction(r, mika, {
      type: "update-settings",
      durationSeconds: 5400,
      bannedCountries: ["fr", "DE", "fr"],
    });
    expect(r.durationSeconds).toBe(5400);
    expect(r.settings).toEqual({
      proximityEnabled: false,
      bannedCountries: ["DE", "FR"],
    });
    expect(r.stepId).not.toBe(step);
    expect(r.players[0].ready).toBe(false);
    const unchanged = r.stepId;
    applyAction(r, "alex", {
      type: "update-settings",
      bannedCountries: ["DE", "FR"],
      durationSeconds: 5400,
    });
    expect(r.stepId).toBe(unchanged);
    expect(() =>
      applyAction(r, "alex", {
        type: "update-settings",
        durationSeconds: 24 * 3600 + 60,
      }),
    ).toThrow("24 hours");
    expect(() =>
      applyAction(r, "alex", {
        type: "update-settings",
        proximityEnabled: true,
        bannedCountries: ["DEU"],
      }),
    ).toThrow();
    expect(r.settings.proximityEnabled).toBe(false);
    readyAll(r);
    expect(() =>
      applyAction(r, "alex", {
        type: "update-settings",
        proximityEnabled: true,
      }),
    ).toThrow("locked");
  });

  it("invalidates category agreement after an edit and prevents an empty search", () => {
    const r = room();
    readyAll(r);
    applyAction(r, "alex", { type: "ready", stepId: r.stepId, ready: true });
    applyAction(r, "alex", {
      type: "edit-category",
      categoryId: r.categories[0].id,
      text: "A blue door",
    });
    expect(r.players.every((p) => !p.ready)).toBe(true);
    r.categories = [];
    expect(() =>
      applyAction(r, "alex", { type: "ready", stepId: r.stepId, ready: true }),
    ).toThrow("category");
  });

  it("expires on one overall deadline without unanimous readiness and rejects late findings", () => {
    const r = searching();
    applyAction(
      r,
      "alex",
      { type: "ready", stepId: r.stepId, ready: true },
      2000,
    );
    expect(r.phase).toBe("searching");
    expect(tick(r, 60999)).toBe(false);
    expect(tick(r, 61000)).toBe(true);
    expect(r.phase).toBe("selection");
    expect(r.players.every((p) => !p.ready)).toBe(true);
    expect(() =>
      applyAction(
        r,
        "alex",
        {
          type: "save-finding",
          categoryId: r.categories[0].id,
          view: demoView(),
        },
        62000,
      ),
    ).toThrow("over");
  });

  it("allows undo during searching and opens selection exactly once when every peer is done", () => {
    const r = searching();
    const stepId = r.stepId;
    applyAction(r, "alex", { type: "ready", stepId, ready: true }, 2000);
    applyAction(r, "alex", { type: "ready", stepId, ready: true }, 2000);
    expect(r.phase).toBe("searching");
    expect(r.searchEndsAt).toBe(61000);
    applyAction(r, "alex", { type: "ready", stepId, ready: false }, 3000);
    for (const player of r.players.slice(1))
      applyAction(r, player.id, { type: "ready", stepId, ready: true }, 4000);
    expect(r.phase).toBe("searching");
    // Alex has no findings; completing every category is optional.
    applyAction(r, "alex", { type: "ready", stepId, ready: true }, 5000);
    expect(r.phase).toBe("selection");
    expect(r.stepId).not.toBe(stepId);
    expect(r.players.every((p) => !p.ready)).toBe(true);
    expect(() =>
      applyAction(r, "alex", { type: "ready", stepId, ready: true }, 6000),
    ).toThrow("moved on");
    expect(() =>
      applyAction(
        r,
        "alex",
        {
          type: "save-finding",
          categoryId: r.categories[0].id,
          view: demoView(),
        },
        6000,
      ),
    ).toThrow("over");
    expect(tick(r, 61000)).toBe(false);
    expect(r.phase).toBe("selection");
  });

  it("resets search readiness when saving evidence and excludes departures while retaining disconnected peers", () => {
    const r = searching();
    const stepId = r.stepId;
    const categoryId = r.categories[0].id;
    applyAction(r, "alex", { type: "ready", stepId, ready: true }, 2000);
    applyAction(
      r,
      "alex",
      { type: "save-finding", categoryId, view: demoView() },
      3000,
    );
    expect(r.players[0].ready).toBe(false);
    applyAction(r, "alex", { type: "ready", stepId, ready: true }, 4000);
    applyAction(
      r,
      "alex",
      {
        type: "delete-finding",
        findingId: r.findings.find((f) => f.playerId === "alex")!.id,
      },
      5000,
    );
    expect(r.players[0].ready).toBe(false);
    r.players[0].ready = r.players[1].ready = true;
    r.players[2].connected = false;
    reconcile(r);
    expect(r.phase).toBe("searching");
    r.players[2].left = true;
    reconcile(r);
    expect(r.phase).toBe("selection");
    expect(r.players.every((p) => !p.ready)).toBe(true);
  });

  it("lets solo demo players finish searching through the same readiness flow", () => {
    const r = createRoom("DEMO", "Alex", "alex", 60, true);
    readyAll(r);
    readyAll(r);
    expect(simulatePeers(r, 2000)).toBe(true);
    expect(r.phase).toBe("searching");
    expect(r.players.filter((p) => p.bot).every((p) => p.ready)).toBe(true);
    applyAction(
      r,
      "alex",
      { type: "ready", stepId: r.stepId, ready: true },
      3000,
    );
    expect(r.phase).toBe("selection");
    expect(r.players.every((p) => !p.ready)).toBe(true);
  });

  it("defaults to the first candidate, handles missing findings implicitly, and repairs deleted selections", () => {
    const r = searching();
    const categoryId = r.categories[0].id;
    applyAction(
      r,
      "alex",
      { type: "save-finding", categoryId, view: demoView() },
      2000,
    );
    applyAction(
      r,
      "alex",
      { type: "save-finding", categoryId, view: demoView(1) },
      3000,
    );
    const candidates = r.findings.filter((f) => f.playerId === "alex");
    expect(r.selections.alex[categoryId]).toBe(candidates[0].id);
    expect(r.selections.alex[r.categories[1].id]).toBeUndefined();
    tick(r, 61000);
    applyAction(
      r,
      "alex",
      { type: "ready", stepId: r.stepId, ready: true },
      62000,
    );
    applyAction(
      r,
      "alex",
      { type: "select", categoryId, findingId: candidates[1].id },
      62000,
    );
    expect(r.players[0].ready).toBe(false);
    applyAction(
      r,
      "alex",
      { type: "delete-finding", findingId: candidates[1].id },
      62000,
    );
    expect(r.selections.alex[categoryId]).toBe(candidates[0].id);
    applyAction(
      r,
      "alex",
      { type: "delete-finding", findingId: candidates[0].id },
      62000,
    );
    expect(r.selections.alex[categoryId]).toBeUndefined();
  });

  it("never includes another player’s candidates or private votes in a snapshot", () => {
    const r = searching();
    expect(r.findings.length).toBeGreaterThan(0);
    expect(snapshot(r, "alex").ownFindings).toEqual([]);
    tick(r, 61000);
    readyAll(r);
    const finding = currentFinding(r)!;
    applyAction(
      r,
      "alex",
      { type: "vote", itemId: finding.id, vote: "reject" },
      62000,
    );
    const other = snapshot(r, r.players[1].id);
    expect(other.review?.ownVote).toBeNull();
    expect(other.review?.result).toBeNull();
    expect(other).not.toHaveProperty("votes");
  });

  it("orders review by category then player and skips absent submissions", () => {
    const r = searching();
    applyAction(
      r,
      "alex",
      {
        type: "save-finding",
        categoryId: r.categories[0].id,
        view: demoView(),
      },
      2000,
    );
    tick(r, 61000);
    readyAll(r);
    expect(
      r.queue.map((id) => r.findings.find((f) => f.id === id)!.categoryId),
    ).toEqual([
      r.categories[0].id,
      r.categories[0].id,
      r.categories[0].id,
      r.categories[1].id,
      r.categories[1].id,
      r.categories[2].id,
      r.categories[2].id,
    ]);
    expect(currentFinding(r)?.playerId).toBe("alex");
  });

  it("counts votes as ready, waits for the owner, treats ties as failure, and advances without a reveal", () => {
    const r = searching();
    r.categories = [r.categories[0]];
    tick(r, 61000);
    readyAll(r);
    const finding = currentFinding(r)!;
    expect(() =>
      applyAction(r, finding.playerId, {
        type: "vote",
        itemId: finding.id,
        vote: "accept",
      }),
    ).toThrow("own");
    expect(() =>
      applyAction(r, "alex", { type: "ready", stepId: r.stepId, ready: true }),
    ).toThrow("Cast your vote");
    for (let i = 0; i < 2; i++) {
      const current = currentFinding(r)!;
      const voters = r.players.filter((p) => p.id !== current.playerId);
      voters.forEach((p, n) =>
        applyAction(r, p.id, {
          type: "vote",
          itemId: current.id,
          vote: n === 0 ? "accept" : "reject",
        }),
      );
      expect(voters.every((p) => p.ready)).toBe(true);
      expect(r.reviewIndex).toBe(i);
      expect(r.results).toHaveLength(i);
      const stepId = r.stepId;
      applyAction(r, current.playerId, { type: "ready", stepId, ready: true });
      expect(r.reviewIndex).toBe(i + 1);
      expect(r.stepId).not.toBe(stepId);
      expect(r.players.every((p) => !p.ready)).toBe(true);
      expect(r.votes).toEqual({});
      expect(r.results.at(-1)?.accepted).toBe(false);
      expect(() =>
        applyAction(r, "alex", {
          type: "vote",
          itemId: current.id,
          vote: "accept",
        }),
      ).toThrow("closed");
      expect(() =>
        applyAction(r, current.playerId, {
          type: "ready",
          stepId,
          ready: true,
        }),
      ).toThrow("moved on");
    }
    expect(r.phase).toBe("finished");
    expect(r.results).toHaveLength(2);
    expect(r.players.every((p) => p.score === 0)).toBe(true);
  });

  it("allows vote changes while waiting and advances on the last vote when the owner is ready", () => {
    const r = searching();
    r.categories = [r.categories[0]];
    tick(r, 61000);
    readyAll(r);
    const finding = currentFinding(r)!;
    const voters = r.players.filter((p) => p.id !== finding.playerId);
    applyAction(r, finding.playerId, {
      type: "ready",
      stepId: r.stepId,
      ready: true,
    });
    applyAction(r, voters[0].id, {
      type: "vote",
      itemId: finding.id,
      vote: "reject",
    });
    applyAction(r, voters[0].id, {
      type: "vote",
      itemId: finding.id,
      vote: "accept",
    });
    expect(voters[0].ready).toBe(true);
    expect(() =>
      applyAction(r, voters[0].id, {
        type: "ready",
        stepId: r.stepId,
        ready: false,
      }),
    ).toThrow("Cast your vote");
    expect(r.reviewIndex).toBe(0);
    applyAction(r, voters[1].id, {
      type: "vote",
      itemId: finding.id,
      vote: "accept",
    });
    expect(r.reviewIndex).toBe(1);
    expect(r.results[0]).toMatchObject({
      itemId: finding.id,
      accepts: 2,
      rejects: 0,
      accepted: true,
    });
    expect(r.players.find((p) => p.id === finding.playerId)?.score).toBe(1);
    expect(() =>
      applyAction(r, voters[1].id, {
        type: "vote",
        itemId: finding.id,
        vote: "accept",
      }),
    ).toThrow("closed");
    expect(r.results).toHaveLength(1);
  });

  it("removes departed players from review requirements and preserves their submissions", () => {
    const r = searching();
    tick(r, 61000);
    readyAll(r);
    const finding = currentFinding(r)!;
    const voters = r.players.filter((p) => p.id !== finding.playerId);
    const owner = r.players.find((p) => p.id === finding.playerId)!;
    owner.left = true;
    applyAction(r, voters[0].id, {
      type: "vote",
      itemId: finding.id,
      vote: "accept",
    });
    expect(r.results).toHaveLength(0);
    voters[1].left = true;
    reconcile(r);
    expect(r.reviewIndex).toBe(1);
    expect(r.results[0]).toMatchObject({
      itemId: finding.id,
      accepted: true,
      accepts: 1,
    });
    expect(owner.score).toBe(1);
  });

  it("keeps disconnected peers in readiness but excludes explicit departures", () => {
    const r = room();
    r.players[2].connected = false;
    r.players[0].ready = r.players[1].ready = true;
    reconcile(r);
    expect(r.phase).toBe("lobby");
    r.players[2].left = true;
    reconcile(r);
    expect(r.phase).toBe("categories");
  });

  it("finishes gracefully if nobody has a finding", () => {
    const r = searching();
    r.findings = [];
    tick(r, 61000);
    readyAll(r);
    expect(r.phase).toBe("finished");
    expect(r.results).toEqual([]);
  });

  it("restarts a finished round as a fresh lobby with the same crew and rules", () => {
    const r = room();
    applyAction(r, "alex", {
      type: "update-settings",
      durationSeconds: 5400,
      bannedCountries: ["FR"],
    });
    expect(() =>
      applyAction(r, "alex", { type: "restart", stepId: r.stepId }),
    ).not.toThrow();
    expect(r.phase).toBe("lobby");
    readyAll(r);
    readyAll(r);
    expect(r.phase).toBe("searching");
    expect(() =>
      applyAction(r, "alex", { type: "restart", stepId: r.stepId }, 2000),
    ).toThrow("moved on");
    applyAction(
      r,
      "alex",
      { type: "save-finding", categoryId: r.categories[0].id, view: demoView() },
      2000,
    );
    tick(r, 1000 + 5400 * 1000);
    readyAll(r);
    while (r.phase === "review") {
      const finding = currentFinding(r)!;
      for (const p of r.players)
        if (p.id !== finding.playerId)
          applyAction(r, p.id, { type: "vote", itemId: finding.id, vote: "accept" });
      applyAction(r, finding.playerId, { type: "ready", stepId: r.stepId, ready: true });
    }
    expect(r.phase).toBe("finished");
    expect(r.players.some((p) => p.score > 0)).toBe(true);
    const finished = r.stepId;
    const mika = r.players[1].id;
    applyAction(r, mika, { type: "restart", stepId: finished });
    // A duplicate click from another player is harmless.
    const lobby = r.stepId;
    applyAction(r, "alex", { type: "restart", stepId: finished });
    expect(r.stepId).toBe(lobby);
    const state = snapshot(r, "alex");
    expect(state.phase).toBe("lobby");
    expect(state.players.map((p) => p.score)).toEqual([0, 0, 0]);
    expect(state.players.every((p) => !p.ready)).toBe(true);
    expect(state.categories).toEqual([]);
    expect(state.ownFindings).toEqual([]);
    expect(state.results).toEqual([]);
    expect(state.searchEndsAt).toBeNull();
    expect(state.durationSeconds).toBe(5400);
    expect(state.settings.bannedCountries).toEqual(["FR"]);
  });
});

describe("search starts", () => {
  it("gives local players distinct starts outside banned countries and keeps the demo illustrated", () => {
    const local = createRoom("LOCAL1", "Alex", "alex", 60, false, {
      proximityEnabled: false,
      bannedCountries: ["DE"],
    });
    local.players.push({ ...local.players[0], id: "mika", name: "Mika" });
    expect(snapshot(local, "alex").start).toBeNull();
    readyAll(local);
    applyAction(local, "alex", { type: "add-category", text: "Door" });
    readyAll(local);
    expect(local.phase).toBe("searching");
    const [alex, mika] = [
      snapshot(local, "alex").start,
      snapshot(local, "mika").start,
    ];
    expect(alex).not.toBeNull();
    expect(alex).not.toEqual(mika);
    expect(alex).not.toEqual({ lat: 52.52, lng: 13.405 });
    expect(snapshot(local, "alex").settings.bannedCountries).toEqual(["DE"]);
    const demo = searching();
    expect(snapshot(demo, "alex").start).toBeNull();
  });

  it("cycles through allowed starts when there are more players than places", () => {
    expect(randomStarts(30, ["DE"])).toHaveLength(30);
    expect(
      randomStarts(3, [
        "DE",
        "FR",
        "GB",
        "IT",
        "JP",
        "US",
        "BR",
        "AU",
        "ZA",
        "CA",
        "MX",
        "TH",
      ]),
    ).toEqual([]);
  });
});
