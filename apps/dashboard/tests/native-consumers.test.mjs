import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";

const build = pathToFileURL(resolve(process.env.CAREOPS_DASHBOARD_BUILD_ROOT ?? "dist") + "/");
const { SimulationWebSocketClient } = await import(new URL("src/live/websocket_client.js", build));
const { SpatialLayoutManager } = await import(new URL("src/spatial/spatial_layout.js", build));
const protocol_version = "careops_ed.abi.v1";
const snapshot = { type: "snapshot", protocol_version, run_id: "run", sequence: 10,
  sim_tick: "9007199254740993", spatial_revision: "synthetic", entities: [], is_terminal: false };
const delta = { type: "delta", protocol_version, run_id: "run", sequence: 11,
  base_sequence: 10, base_tick: snapshot.sim_tick, sim_tick: "9007199254740994", changes: [] };
const send = (client, message) => client.handleIncomingMessage(JSON.stringify(message));

test("commands do not advance the inbound stream; contiguous delta and progress retain wide ticks", () => {
  const client = new SimulationWebSocketClient();
  assert.equal(send(client, snapshot), true);
  const command = client.sendCommand("pause");
  assert.equal(command.type, "command");
  assert.equal(command.run_id, "run");
  assert.equal(command.action, "pause");
  assert.equal(command.client_sequence, 1);
  assert.ok(command.client_nonce);
  assert.ok(command.command_id);
  assert.equal(client.latestSequence, 10);
  assert.equal(send(client, { type: "progress", protocol_version, run_id: "run",
    current_tick: "9007199254740994", horizon_tick: "9007199254741000",
    percentage: 50, status: "running", wall_clock_elapsed_ms: 1 }), true);
  assert.equal(client.latestTick, "9007199254740994");
  assert.equal(client.latestSequence, 10);
  assert.equal(send(client, delta), true);
  assert.equal(client.latestSequence, 11);
  assert.equal(client.latestTick, delta.sim_tick);
  assert.equal(send(client, { type: "progress", protocol_version, run_id: "run",
    current_tick: "9007199254740995", horizon_tick: "9007199254741000",
    percentage: 50, status: "running", wall_clock_elapsed_ms: 1 }), true);
  assert.equal(client.latestTick, "9007199254740995");
});

test("missing, stale, foreign-run and incompatible-base deltas require resync without state advancement", () => {
  for (const invalid of [{ sequence: 12 }, { sequence: 10 }, { run_id: "foreign" },
    { base_sequence: 9 }, { base_tick: "0" }]) {
    const client = new SimulationWebSocketClient();
    send(client, snapshot);
    assert.equal(send(client, { ...delta, ...invalid }), false);
    assert.equal(client.connectionState, "resync_required");
    assert.equal(client.latestSequence, snapshot.sequence);
    assert.equal(client.latestTick, snapshot.sim_tick);
  }
});

test("both retained layout representations and empty routes use consistent edge data", () => {
  const graph = { nodes: [{ node_id: "a", x_mm: 0, y_mm: 0, floor_level: 0 },
    { node_id: "b", x_mm: 3, y_mm: 4, floor_level: 0 }], edges: [
    { edge_id: "ab", from_node: "a", to_node: "b", distance_mm: 5,
      bidirectional: true, speed_limit_mm_per_sec: 1 }] };
  for (const key of ["routes", "route_graph"]) {
    const manager = new SpatialLayoutManager({ locations: [], [key]: graph });
    assert.deepEqual(manager.getAllEdges(), graph.edges);
    assert.equal(manager.findShortestDistance("a", "b"), 5);
  }
  assert.deepEqual(new SpatialLayoutManager({ locations: [] }).getAllEdges(), []);
});

test("package entry and declarations resolve to real build output without retired export APIs", async () => {
  const pkg = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
  assert.equal(pkg.main, "dist/src/index.js");
  assert.equal(pkg.types, "dist/src/index.d.ts");
  const source = await readFile(new URL("src/index.js", build), "utf8");
  const declarations = await readFile(new URL("src/index.d.ts", build), "utf8");
  assert.ok(declarations.includes("LayoutSpatialPackage"));
  assert.ok(!source.includes("scenario_export"));
  const entry = await import(new URL("src/index.js", build));
  assert.equal(typeof entry.SpatialLayoutManager, "function");
  assert.equal(typeof entry.SimulationWebSocketClient, "function");
  assert.equal("getArrowTelemetrySchema" in entry, false);
  assert.equal("exportRunDocumentJson" in entry, false);
  assert.equal("exportComparisonCsv" in entry, false);
  await readFile(new URL("contracts/protocol.js", build));
});
