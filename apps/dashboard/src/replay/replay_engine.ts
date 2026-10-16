/**
 * CareOps ED Simulation Replay Engine (`@careops/dashboard/replay`)
 *
 * Implements frame-by-frame replay of recorded native simulation runs.
 * Strictly separates simulation engine time (discrete ticks) from display frame rates (0/30/60 FPS).
 * Supports scrub/seek without mutating original event ordering or inventing movement.
 */

import { EntitySpatialState, PositionProvenance } from "../spatial/overlays.js";

export interface ReplayFrame {
  tick: number;
  timestamp_sim_str: string;
  entities: EntitySpatialState[];
  active_queue_lengths: Record<string, number>;
  bed_utilization: number;
  event_description?: string;
}

export type PlaybackState = "playing" | "paused" | "completed";

export class SimulationReplayEngine {
  private frames: ReplayFrame[];
  private currentFrameIndex: number;
  private state: PlaybackState;
  private speedMultiplier: number;
  private targetFps: number; // 0 (headless), 30, or 60
  private frameListeners: Array<(frame: ReplayFrame) => void>;

  constructor(frames: ReplayFrame[] = []) {
    this.frames = frames.sort((a, b) => a.tick - b.tick);
    this.currentFrameIndex = 0;
    this.state = "paused";
    this.speedMultiplier = 1.0;
    this.targetFps = 60;
    this.frameListeners = [];
  }

  loadFrames(frames: ReplayFrame[]): void {
    this.frames = frames.sort((a, b) => a.tick - b.tick);
    this.currentFrameIndex = 0;
    this.state = "paused";
    this.notifyCurrentFrame();
  }

  get totalFrames(): number {
    return this.frames.length;
  }

  get currentIndex(): number {
    return this.currentFrameIndex;
  }

  get playbackState(): PlaybackState {
    return this.state;
  }

  get currentTick(): number {
    return this.frames[this.currentFrameIndex]?.tick ?? 0;
  }

  get currentFrame(): ReplayFrame | null {
    return this.frames[this.currentFrameIndex] ?? null;
  }

  setFps(fps: number): void {
    this.targetFps = fps;
  }

  get fps(): number {
    return this.targetFps;
  }

  setSpeed(multiplier: number): void {
    if (multiplier > 0) {
      this.speedMultiplier = multiplier;
    }
  }

  get speed(): number {
    return this.speedMultiplier;
  }

  subscribe(listener: (frame: ReplayFrame) => void): () => void {
    this.frameListeners.push(listener);
    if (this.currentFrame) {
      listener(this.currentFrame);
    }
    return () => {
      this.frameListeners = this.frameListeners.filter((l) => l !== listener);
    };
  }

  play(): void {
    if (this.frames.length === 0) return;
    if (this.currentFrameIndex >= this.frames.length - 1) {
      this.currentFrameIndex = 0;
    }
    this.state = "playing";
  }

  pause(): void {
    this.state = "paused";
  }

  stepForward(): void {
    if (this.currentFrameIndex < this.frames.length - 1) {
      this.currentFrameIndex++;
      this.notifyCurrentFrame();
    } else {
      this.state = "completed";
    }
  }

  stepBackward(): void {
    if (this.currentFrameIndex > 0) {
      this.currentFrameIndex--;
      this.notifyCurrentFrame();
    }
  }

  seekToTick(targetTick: number): boolean {
    if (this.frames.length === 0) return false;

    // Find closest frame <= targetTick
    let bestIndex = 0;
    for (let i = 0; i < this.frames.length; i++) {
      if (this.frames[i].tick <= targetTick) {
        bestIndex = i;
      } else {
        break;
      }
    }

    this.currentFrameIndex = bestIndex;
    this.notifyCurrentFrame();
    return true;
  }

  seekToPercent(percent: number): void {
    if (this.frames.length === 0) return;
    const clamped = Math.max(0, Math.min(1.0, percent));
    this.currentFrameIndex = Math.min(
      this.frames.length - 1,
      Math.floor(clamped * (this.frames.length - 1))
    );
    this.notifyCurrentFrame();
  }

  private notifyCurrentFrame(): void {
    const frame = this.currentFrame;
    if (frame) {
      for (const listener of this.frameListeners) {
        listener(frame);
      }
    }
  }
}
