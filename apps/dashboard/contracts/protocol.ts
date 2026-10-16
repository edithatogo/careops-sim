/**
 * CareOps ED Simulation Dashboard ABI & Spatial Contracts (v1)
 *
 * Freezes the narrow WebSocket ABI snapshot/delta/command contracts and shared spatial
 * package representation between the native/Wasm Kairos simulation engine and browser client.
 *
 * NOTE ON WIDE INTEGERS (u128 / u64):
 * JavaScript Numbers are IEEE 754 64-bit double-precision floats, which lose exact integer
 * precision beyond Number.MAX_SAFE_INTEGER (2^53 - 1 = 9,007,199,254,740,991).
 * High-resolution simulation ticks (SimTime in femtoseconds/nanoseconds/ticks) and 128-bit entity
 * or sequence IDs are strictly serialized as base-10 decimal strings (WideIntegerString).
 * Clients MUST parse them using BigInt(str) and format via bigint.toString().
 */

export const ABI_PROTOCOL_VERSION = "careops_ed.abi.v1" as const;
export const SPATIAL_SCHEMA_VERSION = "careops_ed.spatial.v1" as const;

/**
 * Supported browser targets matrix:
 * Chrome 110+, Firefox 110+, Safari 16.4+, Edge 110+
 * Requirements: WebAssembly, BigInt, WebSocket, ResizeObserver, OffscreenCanvas.
 */
export const SUPPORTED_BROWSER_TARGETS = {
  chrome: ">=110",
  firefox: ">=110",
  safari: ">=16.4",
  edge: ">=110",
} as const;

/** Buffer and queue limits to prevent memory exhaustion in browser runtime. */
export const BUFFER_LIMITS = {
  MAX_ENVELOPE_BYTES: 1024 * 1024, // 1 MB maximum single message size
  MAX_QUEUE_DEPTH: 1000,           // Maximum unconsumed envelopes before backpressure/drop
  BOUNDED_RING_BUFFER_SLOTS: 512,  // Retained delta history for reconnect resync
} as const;

/** Base-10 unsigned integer string representation for u64 / u128 types. */
export type WideIntegerString = string;

export function isWideIntegerString(value: unknown): value is WideIntegerString {
  return typeof value === "string" && /^[0-9]+$/.test(value);
}

export function parseWideInteger(value: WideIntegerString): bigint {
  if (!isWideIntegerString(value)) {
    throw new TypeError(`Invalid WideIntegerString: "${value}"`);
  }
  return BigInt(value);
}

export function formatWideInteger(value: bigint | number): WideIntegerString {
  if (typeof value === "number") {
    if (!Number.isInteger(value) || value < 0) {
      throw new RangeError(`Value must be a non-negative integer: ${value}`);
    }
    return Math.floor(value).toString();
  }
  if (typeof value === "bigint") {
    if (value < 0n) {
      throw new RangeError(`BigInt value must be non-negative: ${value}`);
    }
    return value.toString();
  }
  throw new TypeError(`Expected bigint or number, got ${typeof value}`);
}

// ============================================================================
// ABI Envelope Types
// ============================================================================

export interface CoordinatesMm {
  x_mm: number;
  y_mm: number;
  z_mm?: number;
}

export interface EntityState {
  entity_id: string;
  kind: string;
  location_id: string;
  status: string;
  coordinates_mm?: CoordinatesMm;
  attributes?: Record<string, unknown>;
}

export interface EntityChange {
  entity_id: string;
  op: "spawn" | "update" | "despawn" | "move";
  location_id?: string;
  coordinates_mm?: CoordinatesMm;
  status?: string;
  attributes?: Record<string, unknown>;
}

export interface SimEvent {
  event_type: string;
  tick: WideIntegerString;
  entity_id?: string;
  payload?: Record<string, unknown>;
}

/** Full state snapshot envelope sent initially or upon resync. */
export interface SnapshotEnvelope {
  type: "snapshot";
  protocol_version: typeof ABI_PROTOCOL_VERSION;
  run_id: string;
  sequence: number;
  sim_tick: WideIntegerString;
  spatial_revision: string;
  entities: EntityState[];
  metrics?: Record<string, unknown>;
  queue_states?: Record<string, unknown>;
  is_terminal: boolean;
}

/** Incremental state update envelope applied sequentially. */
export interface DeltaEnvelope {
  type: "delta";
  protocol_version: typeof ABI_PROTOCOL_VERSION;
  run_id: string;
  sequence: number;
  base_sequence: number;
  sim_tick: WideIntegerString;
  base_tick: WideIntegerString;
  changes: EntityChange[];
  events?: SimEvent[];
}

export type CommandAction =
  | "pause"
  | "resume"
  | "step"
  | "set_speed"
  | "cancel"
  | "inject_patient"
  | "divert_ambulance";

/** Client-to-server command envelope with idempotency token. */
export interface CommandEnvelope {
  type: "command";
  protocol_version: typeof ABI_PROTOCOL_VERSION;
  command_id: string;
  client_nonce: string;
  run_id: string;
  client_sequence: number;
  action: CommandAction;
  target_tick?: WideIntegerString;
  payload?: Record<string, unknown>;
}

export type CommandAckStatus = "accepted" | "rejected" | "applied" | "duplicate_noop";

/** Server acknowledgement for a client command. */
export interface CommandAckEnvelope {
  type: "command_ack";
  protocol_version: typeof ABI_PROTOCOL_VERSION;
  command_id: string;
  client_nonce: string;
  run_id: string;
  status: CommandAckStatus;
  effective_tick: WideIntegerString;
  error_message?: string | null;
}

export type ProgressStatus = "running" | "paused" | "completed" | "cancelled" | "diverted";

/** Execution progress heartbeat. */
export interface ProgressEnvelope {
  type: "progress";
  protocol_version: typeof ABI_PROTOCOL_VERSION;
  run_id: string;
  current_tick: WideIntegerString;
  horizon_tick: WideIntegerString;
  percentage: number;
  status: ProgressStatus;
  wall_clock_elapsed_ms: number;
}

export type ErrorCode =
  | "SEQUENCE_GAP"
  | "STALE_RUN_ID"
  | "IDEMPOTENCY_CONFLICT"
  | "INVALID_COMMAND"
  | "BUFFER_OVERFLOW"
  | "REVISION_MISMATCH"
  | "INTERNAL_ERROR";

/** Error notification envelope. */
export interface ErrorEnvelope {
  type: "error";
  protocol_version: typeof ABI_PROTOCOL_VERSION;
  run_id?: string;
  error_code: ErrorCode;
  message: string;
  details?: Record<string, unknown>;
  occurred_at_tick?: WideIntegerString;
}

export type AbiEnvelope =
  | SnapshotEnvelope
  | DeltaEnvelope
  | CommandEnvelope
  | CommandAckEnvelope
  | ProgressEnvelope
  | ErrorEnvelope;

// ============================================================================
// Spatial Package Types (careops_ed.spatial.v1)
// ============================================================================

export type SpatialLocationKind =
  | "triage"
  | "acute_bay"
  | "resus"
  | "waiting_room"
  | "corridor";

export interface SpatialBounds {
  min_x: number;
  min_y: number;
  max_x: number;
  max_y: number;
}

export interface SpatialLocation {
  location_id: string;
  name: string;
  zone: string;
  kind: SpatialLocationKind;
  coordinates: CoordinatesMm;
  bounds: SpatialBounds;
}

export interface RouteNode {
  node_id: string;
  location_id: string;
  x_mm: number;
  y_mm: number;
  z_mm?: number;
}

export interface RouteEdge {
  edge_id?: string;
  from_node: string;
  to_node: string;
  distance_mm: number;
  bidirectional: boolean;
  speed_limit_mm_per_sec: number;
}

export interface SpatialRoutes {
  nodes: RouteNode[];
  edges: RouteEdge[];
}

export interface SpatialPackage {
  schema_version: typeof SPATIAL_SCHEMA_VERSION;
  spatial_revision: string;
  units: "millimeters";
  coordinate_system: "right-handed Cartesian 2D/3D" | "right-handed Cartesian 2D" | "right-handed Cartesian 3D";
  scale_factor_to_meters: 0.001;
  locations: SpatialLocation[] | Record<string, SpatialLocation>;
  routes: SpatialRoutes | RouteEdge[];
}

// ============================================================================
// Reconnect / Resync State Tracking
// ============================================================================

export interface ClientSyncState {
  run_id: string | null;
  last_received_sequence: number;
  last_received_tick: WideIntegerString | null;
  spatial_revision: string | null;
  resync_in_progress: boolean;
}

export interface ResyncRequestPayload {
  run_id: string;
  last_sequence: number;
  reason: "gap_detected" | "client_reconnect" | "stale_state";
}
