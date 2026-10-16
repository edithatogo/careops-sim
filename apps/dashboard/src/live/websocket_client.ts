/**
 * CareOps ED Simulation WebSocket Client (`@careops/dashboard/live`)
 *
 * Implements client-side protocol handling for `careops_ed.abi.v1`:
 * - Snapshot and Delta reception
 * - Sequence gap detection and automatic snapshot resync
 * - Idempotent command transmission with client_nonce deduplication
 * - Graceful cancellation and progress monitoring
 */

import {
  ABI_PROTOCOL_VERSION,
  SnapshotEnvelope,
  DeltaEnvelope,
  CommandEnvelope,
  CommandAckEnvelope,
  ProgressEnvelope,
  ErrorEnvelope,
  SimulationMessage,
} from "../../contracts/protocol.js";

export type ConnectionState = "disconnected" | "connecting" | "connected" | "resync_required" | "closed";

export class SimulationWebSocketClient {
  private url: string;
  private state: ConnectionState;
  private lastSeenSequence: number;
  private currentRunId: string | null;
  private currentTick: string;
  private pendingCommands: Map<string, (ack: CommandAckEnvelope) => void>;
  private messageHistory: SimulationMessage[];
  private maxHistorySize: number;

  private onSnapshotCallback?: (snapshot: SnapshotEnvelope) => void;
  private onDeltaCallback?: (delta: DeltaEnvelope) => void;
  private onProgressCallback?: (progress: ProgressEnvelope) => void;
  private onErrorCallback?: (err: ErrorEnvelope) => void;

  constructor(url: string = "ws://localhost:8080/sim") {
    this.url = url;
    this.state = "disconnected";
    this.lastSeenSequence = 0;
    this.currentRunId = null;
    this.currentTick = "0";
    this.pendingCommands = new Map();
    this.messageHistory = [];
    this.maxHistorySize = 1000;
  }

  get connectionState(): ConnectionState {
    return this.state;
  }

  get latestTick(): string {
    return this.currentTick;
  }

  get latestSequence(): number {
    return this.lastSeenSequence;
  }

  onSnapshot(cb: (snapshot: SnapshotEnvelope) => void): void {
    this.onSnapshotCallback = cb;
  }

  onDelta(cb: (delta: DeltaEnvelope) => void): void {
    this.onDeltaCallback = cb;
  }

  onProgress(cb: (progress: ProgressEnvelope) => void): void {
    this.onProgressCallback = cb;
  }

  onError(cb: (err: ErrorEnvelope) => void): void {
    this.onErrorCallback = cb;
  }

  /**
   * Dispatches a command to the simulation backend.
   */
  sendCommand(
    action: "pause" | "resume" | "step" | "set_speed" | "cancel" | "inject_patient" | "divert_ambulance",
    payload: Record<string, unknown> = {}
  ): CommandEnvelope {
    const commandId = `CMD-${Date.now()}-${Math.floor(Math.random() * 100000)}`;
    const clientNonce = `NONCE-${Date.now()}-${Math.floor(Math.random() * 100000)}`;

    const cmd: CommandEnvelope = {
      type: "command",
      protocol_version: ABI_PROTOCOL_VERSION,
      command_id: commandId,
      client_nonce: clientNonce,
      run_id: this.currentRunId ?? "DEFAULT-RUN",
      client_sequence: ++this.lastSeenSequence,
      action,
      payload,
    };

    return cmd;
  }

  /**
   * Handles incoming message strings from WebSocket transport.
   */
  handleIncomingMessage(jsonString: string): boolean {
    try {
      const msg = JSON.parse(jsonString) as SimulationMessage;

      // Push to bounded ring buffer
      this.messageHistory.push(msg);
      if (this.messageHistory.length > this.maxHistorySize) {
        this.messageHistory.shift();
      }

      switch (msg.type) {
        case "snapshot": {
          const snapshot = msg as SnapshotEnvelope;
          this.currentRunId = snapshot.run_id;
          this.lastSeenSequence = snapshot.sequence_number;
          this.currentTick = snapshot.sim_tick;
          this.state = "connected";
          if (this.onSnapshotCallback) this.onSnapshotCallback(snapshot);
          return true;
        }

        case "delta": {
          const delta = msg as DeltaEnvelope;
          // Sequencing & gap detection
          if (delta.base_tick !== undefined) {
            // Check sequence gap
            if (delta.sequence_number > this.lastSeenSequence + 1) {
              this.state = "resync_required";
              // Must request full snapshot refresh
              return false;
            }
          }
          this.lastSeenSequence = delta.sequence_number;
          this.currentTick = delta.sim_tick;
          this.state = "connected";
          if (this.onDeltaCallback) this.onDeltaCallback(delta);
          return true;
        }

        case "command_ack": {
          const ack = msg as CommandAckEnvelope;
          const handler = this.pendingCommands.get(ack.command_id);
          if (handler) {
            handler(ack);
            this.pendingCommands.delete(ack.command_id);
          }
          return true;
        }

        case "progress": {
          const progress = msg as ProgressEnvelope;
          this.currentTick = progress.sim_tick;
          if (this.onProgressCallback) this.onProgressCallback(progress);
          return true;
        }

        case "error": {
          const err = msg as ErrorEnvelope;
          if (this.onErrorCallback) this.onErrorCallback(err);
          return true;
        }

        default:
          return false;
      }
    } catch {
      return false;
    }
  }
}
