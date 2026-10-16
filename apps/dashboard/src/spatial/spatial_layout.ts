/**
 * CareOps ED Spatial Layout Manager (`@careops/dashboard/spatial`)
 *
 * Implements layout loading, coordinate normalization, topological verification,
 * and shortest-path calculation over the shared spatial package (`careops_ed.spatial.v1`).
 */

export interface Coordinate2D {
  x_mm: number;
  y_mm: number;
}

export interface Coordinate3D extends Coordinate2D {
  z_mm: number;
}

export interface LocationBounds {
  min_x: number;
  min_y: number;
  max_x: number;
  max_y: number;
}

export interface SpatialLocation {
  location_id: string;
  name: string;
  zone: string;
  kind: string;
  floor_level: number;
  coordinates: Coordinate3D;
  bounds: LocationBounds;
}

export interface RouteNode {
  node_id: string;
  floor_level: number;
  x_mm: number;
  y_mm: number;
}

export interface RouteEdge {
  edge_id: string;
  from_node: string;
  to_node: string;
  distance_mm: number;
  bidirectional: boolean;
  speed_limit_mm_per_sec: number;
}

export interface SpatialPackage {
  schema_version: string;
  spatial_revision: string;
  name: string;
  units: string;
  coordinate_system: string;
  scale_factor_to_meters: number;
  bounding_box: {
    min_x_mm: number;
    min_y_mm: number;
    max_x_mm: number;
    max_y_mm: number;
  };
  floor_levels: Array<{ level: number; name: string; elevation_mm: number }>;
  locations: SpatialLocation[];
  route_graph?: {
    nodes: RouteNode[];
    edges: RouteEdge[];
  };
  routes?: {
    nodes: RouteNode[];
    edges: RouteEdge[];
  };
}

export class SpatialLayoutManager {
  private pkg: SpatialPackage;
  private locationMap: Map<string, SpatialLocation>;
  private nodeMap: Map<string, RouteNode>;
  private adjacency: Map<string, Array<{ to: string; dist_mm: number }>>;

  constructor(pkg: SpatialPackage) {
    this.pkg = pkg;
    this.locationMap = new Map();
    this.nodeMap = new Map();
    this.adjacency = new Map();

    for (const loc of pkg.locations) {
      this.locationMap.set(loc.location_id, loc);
    }

    const routeData = pkg.routes || pkg.route_graph || { nodes: [], edges: [] };

    for (const node of routeData.nodes) {
      this.nodeMap.set(node.node_id, node);
      this.adjacency.set(node.node_id, []);
    }

    for (const edge of routeData.edges) {
      const fromList = this.adjacency.get(edge.from_node);
      if (fromList) {
        fromList.push({ to: edge.to_node, dist_mm: edge.distance_mm });
      }
      if (edge.bidirectional) {
        const toList = this.adjacency.get(edge.to_node);
        if (toList) {
          toList.push({ to: edge.from_node, dist_mm: edge.distance_mm });
        }
      }
    }
  }

  get revision(): string {
    return this.pkg.spatial_revision;
  }

  get packageData(): SpatialPackage {
    return this.pkg;
  }

  getLocation(locationId: string): SpatialLocation | undefined {
    return this.locationMap.get(locationId);
  }

  getAllLocations(): SpatialLocation[] {
    return Array.from(this.locationMap.values());
  }

  getNode(nodeId: string): RouteNode | undefined {
    return this.nodeMap.get(nodeId);
  }

  getAllNodes(): RouteNode[] {
    return Array.from(this.nodeMap.values());
  }

  getAllEdges(): RouteEdge[] {
    return this.pkg.route_graph.edges;
  }

  /**
   * Computes shortest path distance in millimeters between two route nodes using Dijkstra.
   */
  findShortestDistance(startNode: string, endNode: string): number | null {
    if (startNode === endNode) return 0;
    if (!this.nodeMap.has(startNode) || !this.nodeMap.has(endNode)) return null;

    const distances = new Map<string, number>();
    const visited = new Set<string>();

    for (const nodeId of this.nodeMap.keys()) {
      distances.set(nodeId, Infinity);
    }
    distances.set(startNode, 0);

    while (visited.size < this.nodeMap.size) {
      let minNode: string | null = null;
      let minDist = Infinity;

      for (const [nodeId, dist] of distances.entries()) {
        if (!visited.has(nodeId) && dist < minDist) {
          minDist = dist;
          minNode = nodeId;
        }
      }

      if (!minNode || minDist === Infinity) break;
      if (minNode === endNode) return minDist;

      visited.add(minNode);

      const neighbors = this.adjacency.get(minNode) || [];
      for (const edge of neighbors) {
        if (!visited.has(edge.to)) {
          const newDist = minDist + edge.dist_mm;
          if (newDist < (distances.get(edge.to) ?? Infinity)) {
            distances.set(edge.to, newDist);
          }
        }
      }
    }

    const result = distances.get(endNode);
    return result !== undefined && result !== Infinity ? result : null;
  }
}
