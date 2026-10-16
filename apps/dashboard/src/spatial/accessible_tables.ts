/**
 * CareOps ED Accessible Tables (`@careops/dashboard/spatial`)
 *
 * Generates screen-reader-accessible HTML tables conforming to WCAG 2.1 AA
 * providing exact equivalents for all spatial view overlays.
 */

import { RoomOccupancyOverlay, QueueOverlay, RouteTransitOverlay, EntitySpatialState } from "./overlays.js";

/**
 * Renders an accessible HTML table for room occupancy overlays.
 */
export function renderAccessibleOccupancyTable(overlays: RoomOccupancyOverlay[]): string {
  const rows = overlays.map((o) => `
    <tr>
      <th scope="row">${o.location_id}</th>
      <td>${o.name}</td>
      <td>${o.capacity}</td>
      <td>${o.current_occupancy}</td>
      <td>${(o.utilization_ratio * 100).toFixed(1)}%</td>
      <td>${o.occupant_entity_ids.length > 0 ? o.occupant_entity_ids.join(", ") : "None"}</td>
    </tr>`).join("");

  return `
<table class="careops-table" role="region" aria-label="Room Occupancy and Utilization">
  <caption>Emergency Department Room Occupancy and Capacity Table</caption>
  <thead>
    <tr>
      <th scope="col">Location ID</th>
      <th scope="col">Room Name</th>
      <th scope="col">Bed Capacity</th>
      <th scope="col">Current Occupancy</th>
      <th scope="col">Utilization</th>
      <th scope="col">Occupant Patient IDs</th>
    </tr>
  </thead>
  <tbody>${rows}
  </tbody>
</table>`.trim();
}

/**
 * Renders an accessible HTML table for waiting queues.
 */
export function renderAccessibleQueueTable(queues: QueueOverlay[]): string {
  const rows = queues.map((q) => `
    <tr>
      <th scope="row">${q.queue_id}</th>
      <td>${q.zone}</td>
      <td>${q.length}</td>
      <td>${q.waiting_patient_ids.length > 0 ? q.waiting_patient_ids.join(", ") : "Empty"}</td>
    </tr>`).join("");

  return `
<table class="careops-table" role="region" aria-label="Waiting Queues Status">
  <caption>Emergency Department Waiting Queues by Zone</caption>
  <thead>
    <tr>
      <th scope="col">Queue ID</th>
      <th scope="col">Zone</th>
      <th scope="col">Queue Depth</th>
      <th scope="col">Waiting Patients</th>
    </tr>
  </thead>
  <tbody>${rows}
  </tbody>
</table>`.trim();
}

/**
 * Renders an accessible HTML table for active entity spatial tracking with provenance.
 */
export function renderAccessibleEntityTable(entities: EntitySpatialState[]): string {
  const rows = entities.map((e) => `
    <tr>
      <th scope="row">${e.entity_id}</th>
      <td>${e.label}</td>
      <td>${e.kind}</td>
      <td>${e.location_id ?? "Unknown / En Route"}</td>
      <td>${e.x_mm !== null && e.y_mm !== null ? `(${e.x_mm}, ${e.y_mm})` : "Unknown"}</td>
      <td>Level ${e.floor_level}</td>
      <td>${e.state}</td>
      <td><span class="provenance-tag ${e.provenance}">${e.provenance}</span></td>
    </tr>`).join("");

  return `
<table class="careops-table" role="region" aria-label="Active Entities and Provenance">
  <caption>Active Clinical Entities, Spatial Coordinates, and Data Provenance</caption>
  <thead>
    <tr>
      <th scope="col">Entity ID</th>
      <th scope="col">Label</th>
      <th scope="col">Kind</th>
      <th scope="col">Location ID</th>
      <th scope="col">Coordinates (mm)</th>
      <th scope="col">Floor Level</th>
      <th scope="col">Status</th>
      <th scope="col">Position Provenance</th>
    </tr>
  </thead>
  <tbody>${rows}
  </tbody>
</table>`.trim();
}
