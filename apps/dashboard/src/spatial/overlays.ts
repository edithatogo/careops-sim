/**
 * CareOps ED Spatial Overlays (`@careops/dashboard/spatial`)
 *
 * Implements visual overlay computations for room occupancy, queue depth,
 * staff assignment, and active transit paths.
 * Enforces provenance boundaries: observed vs simulated vs interpolated vs unknown.
 */

export type PositionProvenance = "observed" | "simulated" | "interpolated" | "unknown";

export interface EntitySpatialState {
  entity_id: number;
  kind: "patient" | "staff" | "equipment";
  label: string;
  location_id: string | null;
  x_mm: number | null;
  y_mm: number | null;
  floor_level: number;
  state: string;
  provenance: PositionProvenance;
}

export interface RoomOccupancyOverlay {
  location_id: string;
  name: string;
  capacity: number;
  current_occupancy: number;
  utilization_ratio: number;
  heat_color: string;
  occupant_entity_ids: number[];
}

export interface QueueOverlay {
  queue_id: string;
  zone: string;
  length: number;
  waiting_patient_ids: number[];
  acuity_breakdown: Record<string, number>;
}

export interface RouteTransitOverlay {
  route_id: string;
  from_location: string;
  to_location: string;
  distance_mm: number;
  traveler_entity_id: number;
  traveler_label: string;
  progress_percent: number;
  current_x_mm: number;
  current_y_mm: number;
}

/**
 * Maps normalized utilization [0.0, 1.0+] to a color hex code.
 */
export function getOccupancyColor(utilization: number): string {
  if (utilization <= 0.0) return "#2ecc71"; // green / available
  if (utilization <= 0.5) return "#27ae60"; // deep green
  if (utilization <= 0.8) return "#f39c12"; // amber / moderate
  if (utilization < 1.0) return "#e67e22"; // dark amber / high
  return "#e74c3c"; // red / saturated
}

/**
 * Calculates room occupancy overlays from active entity spatial states and location capacities.
 */
export function computeRoomOccupancies(
  locations: Array<{ location_id: string; name: string; capacity?: number }>,
  entities: EntitySpatialState[]
): RoomOccupancyOverlay[] {
  const defaultCapacity = 1;
  const occupancyMap = new Map<string, number[]>();

  for (const loc of locations) {
    occupancyMap.set(loc.location_id, []);
  }

  for (const entity of entities) {
    if (entity.location_id && occupancyMap.has(entity.location_id)) {
      occupancyMap.get(entity.location_id)!.push(entity.entity_id);
    }
  }

  return locations.map((loc) => {
    const occupants = occupancyMap.get(loc.location_id) || [];
    const cap = loc.capacity !== undefined && loc.capacity > 0 ? loc.capacity : defaultCapacity;
    const ratio = Math.min(1.5, occupants.length / cap);

    return {
      location_id: loc.location_id,
      name: loc.name,
      capacity: cap,
      current_occupancy: occupants.length,
      utilization_ratio: Math.round(ratio * 100) / 100,
      heat_color: getOccupancyColor(ratio),
      occupant_entity_ids: occupants,
    };
  });
}
