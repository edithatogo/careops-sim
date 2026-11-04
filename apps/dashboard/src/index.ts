/**
 * CareOps ED Dashboard & Spatial Visualization Library (`@careops/dashboard`)
 *
 * Unified exports for scenario editing, comparison,
 * spatial floor plans, accessible tables, replay engine, and live WebSocket telemetry.
 */

export * from "../contracts/protocol.js";

// Scenario module
export * from "./scenario/scenario_editor.js";
export * from "./scenario/scenario_runner.js";

// Spatial module
export { SpatialLayoutManager } from "./spatial/spatial_layout.js";
export type {
  Coordinate2D, Coordinate3D, LocationBounds,
  SpatialPackage as LayoutSpatialPackage,
  SpatialLocation as LayoutSpatialLocation,
  RouteNode as LayoutRouteNode,
  RouteEdge as LayoutRouteEdge,
} from "./spatial/spatial_layout.js";
export * from "./spatial/overlays.js";
export * from "./spatial/accessible_tables.js";
export * from "./spatial/spatial_view.js";

// Replay module
export * from "./replay/replay_engine.js";

// Live telemetry module
export * from "./live/websocket_client.js";
