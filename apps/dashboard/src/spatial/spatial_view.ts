/**
 * CareOps ED Spatial View (`@careops/dashboard/spatial`)
 *
 * Implements 2D floor plan rendering with PixiJS canvas support and accessible SVG fallback.
 * Maps millimeter world coordinates to viewport pixels, provides interactive room/agent selection,
 * and renders heatmaps, queues, and route transit polylines.
 */

import { SpatialLayoutManager, SpatialLocation, RouteEdge, RouteNode } from "./spatial_layout.js";
import { RoomOccupancyOverlay, EntitySpatialState, computeRoomOccupancies } from "./overlays.js";

export interface ViewportTransform {
  scale: number; // pixels per millimeter
  offset_x: number;
  offset_y: number;
}

export class SpatialViewController {
  private layout: SpatialLayoutManager;
  private currentFloor: number;
  private selectedLocationId: string | null;
  private selectedEntityId: number | null;
  private transform: ViewportTransform;
  private occupancyOverlays: RoomOccupancyOverlay[];
  private activeEntities: EntitySpatialState[];

  constructor(layout: SpatialLayoutManager) {
    this.layout = layout;
    this.currentFloor = 0;
    this.selectedLocationId = null;
    this.selectedEntityId = null;
    this.occupancyOverlays = [];
    this.activeEntities = [];

    // Default viewport scaling: 16000mm width mapped to 800px => 0.05 px/mm
    this.transform = {
      scale: 0.05,
      offset_x: 50,
      offset_y: 50,
    };

    this.recomputeOverlays();
  }

  setFloor(floor: number): void {
    this.currentFloor = floor;
  }

  get floor(): number {
    return this.currentFloor;
  }

  selectLocation(locationId: string | null): void {
    this.selectedLocationId = locationId;
  }

  get selectedLocation(): string | null {
    return this.selectedLocationId;
  }

  selectEntity(entityId: number | null): void {
    this.selectedEntityId = entityId;
  }

  get selectedEntity(): number | null {
    return this.selectedEntityId;
  }

  updateEntities(entities: EntitySpatialState[]): void {
    this.activeEntities = entities;
    this.recomputeOverlays();
  }

  private recomputeOverlays(): void {
    const locations = this.layout.getAllLocations();
    this.occupancyOverlays = computeRoomOccupancies(locations, this.activeEntities);
  }

  worldToScreen(x_mm: number, y_mm: number): { x: number; y: number } {
    return {
      x: this.transform.offset_x + x_mm * this.transform.scale,
      y: this.transform.offset_y + y_mm * this.transform.scale,
    };
  }

  /**
   * Generates clean SVG markup representation of the spatial layout and active overlays.
   * Serves as standard SVG rendering and fallback for headless or non-WebGL environments.
   */
  renderSvgMarkup(width: number = 900, height: number = 600): string {
    const locations = this.layout.getAllLocations().filter((loc) => loc.floor_level === this.currentFloor);
    const edges = this.layout.getAllEdges();
    const nodes = this.layout.getAllNodes().filter((n) => n.floor_level === this.currentFloor);

    // 1. Draw route edges
    const edgeLines = edges.map((edge) => {
      const fromNode = this.layout.getNode(edge.from_node);
      const toNode = this.layout.getNode(edge.to_node);
      if (!fromNode || !toNode) return "";
      const p1 = this.worldToScreen(fromNode.x_mm, fromNode.y_mm);
      const p2 = this.worldToScreen(toNode.x_mm, toNode.y_mm);
      return `<line x1="${p1.x}" y1="${p1.y}" x2="${p2.x}" y2="${p2.y}" stroke="#7f8c8d" stroke-width="2" stroke-dasharray="4" />`;
    }).join("\n    ");

    // 2. Draw room boundaries and heatmaps
    const roomRects = locations.map((loc) => {
      const p = this.worldToScreen(loc.bounds.min_x, loc.bounds.min_y);
      const w = (loc.bounds.max_x - loc.bounds.min_x) * this.transform.scale;
      const h = (loc.bounds.max_y - loc.bounds.min_y) * this.transform.scale;
      const overlay = this.occupancyOverlays.find((o) => o.location_id === loc.location_id);
      const fillColor = overlay ? overlay.heat_color : "#ecf0f1";
      const isSelected = this.selectedLocationId === loc.location_id;
      const strokeColor = isSelected ? "#3498db" : "#2c3e50";
      const strokeWidth = isSelected ? 4 : 2;

      return `
    <g class="room-group" data-location-id="${loc.location_id}">
      <rect x="${p.x}" y="${p.y}" width="${w}" height="${h}" fill="${fillColor}" fill-opacity="0.35" stroke="${strokeColor}" stroke-width="${strokeWidth}" rx="4" />
      <text x="${p.x + 8}" y="${p.y + 20}" font-family="sans-serif" font-size="12" font-weight="bold" fill="#2c3e50">${loc.name}</text>
      <text x="${p.x + 8}" y="${p.y + 36}" font-family="sans-serif" font-size="10" fill="#7f8c8d">${overlay?.current_occupancy ?? 0} / ${overlay?.capacity ?? 1} beds</text>
    </g>`;
    }).join("");

    // 3. Draw active entities
    const entityMarkers = this.activeEntities
      .filter((e) => e.floor_level === this.currentFloor && e.x_mm !== null && e.y_mm !== null)
      .map((e) => {
        const pt = this.worldToScreen(e.x_mm!, e.y_mm!);
        const color = e.kind === "patient" ? "#e74c3c" : "#2980b9";
        return `<circle cx="${pt.x}" cy="${pt.y}" r="6" fill="${color}" stroke="#ffffff" stroke-width="2"><title>${e.label} (${e.state})</title></circle>`;
      }).join("\n    ");

    return `
<svg viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Emergency Department Spatial Floor Plan">
  <defs>
    <filter id="shadow" x="-5%" y="-5%" width="110%" height="110%">
      <feDropShadow dx="1" dy="2" stdDeviation="2" flood-opacity="0.15" />
    </filter>
  </defs>
  <rect width="100%" height="100%" fill="#f8f9fa" />
  <g class="routes-layer">
    ${edgeLines}
  </g>
  <g class="rooms-layer">
    ${roomRects}
  </g>
  <g class="entities-layer">
    ${entityMarkers}
  </g>
</svg>`.trim();
  }
}
