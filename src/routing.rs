// src/routing.rs
use crate::ast::RoutingType;
use crate::igr::NodeData;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Represents a point in 2D space
pub type Point = (f64, f64);

/// A* pathfinding node
#[derive(Debug, Clone, PartialEq)]
struct AStarNode {
    pos: (i32, i32),
    g_cost: f64, // Cost from start
    h_cost: f64, // Heuristic cost to end
    f_cost: f64, // Total cost (g + h)
    parent: Option<(i32, i32)>,
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents an obstacle (rectangle) for pathfinding
#[derive(Debug, Clone)]
struct Obstacle {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Obstacle {
    fn new(node: &NodeData) -> Self {
        Self {
            x: node.x.round() as i32,
            y: node.y.round() as i32,
            width: node.width.round() as i32,
            height: node.height.round() as i32,
        }
    }

    fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn intersects_line(&self, start: (i32, i32), end: (i32, i32)) -> bool {
        // Simple line-rectangle intersection check
        // Expand obstacle slightly for safety margin
        let margin = 10;
        let expanded_x1 = self.x - margin;
        let expanded_y1 = self.y - margin;
        let expanded_x2 = self.x + self.width + margin;
        let expanded_y2 = self.y + self.height + margin;

        // Check if line passes through expanded rectangle
        self.line_intersects_rect(
            start,
            end,
            expanded_x1,
            expanded_y1,
            expanded_x2,
            expanded_y2,
        )
    }

    fn line_intersects_rect(
        &self,
        start: (i32, i32),
        end: (i32, i32),
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> bool {
        // Check if either endpoint is inside the rectangle
        if (start.0 >= x1 && start.0 <= x2 && start.1 >= y1 && start.1 <= y2)
            || (end.0 >= x1 && end.0 <= x2 && end.1 >= y1 && end.1 <= y2)
        {
            return true;
        }

        // Check line-segment intersection with rectangle edges
        self.lines_intersect(start, end, (x1, y1), (x2, y1))
            || self.lines_intersect(start, end, (x2, y1), (x2, y2))
            || self.lines_intersect(start, end, (x2, y2), (x1, y2))
            || self.lines_intersect(start, end, (x1, y2), (x1, y1))
    }

    fn lines_intersect(
        &self,
        p1: (i32, i32),
        p2: (i32, i32),
        p3: (i32, i32),
        p4: (i32, i32),
    ) -> bool {
        let d1 = self.direction(p3, p4, p1);
        let d2 = self.direction(p3, p4, p2);
        let d3 = self.direction(p1, p2, p3);
        let d4 = self.direction(p1, p2, p4);

        if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0))
        {
            return true;
        }

        if d1 == 0 && self.on_segment(p3, p1, p4) {
            return true;
        }
        if d2 == 0 && self.on_segment(p3, p2, p4) {
            return true;
        }
        if d3 == 0 && self.on_segment(p1, p3, p2) {
            return true;
        }
        if d4 == 0 && self.on_segment(p1, p4, p2) {
            return true;
        }

        false
    }

    fn direction(&self, a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i32 {
        (c.1 - a.1) * (b.0 - a.0) - (b.1 - a.1) * (c.0 - a.0)
    }

    fn on_segment(&self, a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
        b.0 <= std::cmp::max(a.0, c.0)
            && b.0 >= std::cmp::min(a.0, c.0)
            && b.1 <= std::cmp::max(a.1, c.1)
            && b.1 >= std::cmp::min(a.1, c.1)
    }
}

/// Routing algorithms for edge connections
pub struct EdgeRouter;

impl EdgeRouter {
    /// Generate route points for an edge based on the routing type
    pub fn route_edge(
        start: Point,
        end: Point,
        source_node: &NodeData,
        target_node: &NodeData,
        routing_type: Option<RoutingType>,
    ) -> Vec<[i32; 2]> {
        let routing = routing_type.unwrap_or(RoutingType::Auto);

        match routing {
            RoutingType::Straight => Self::straight_route(start, end),
            RoutingType::Orthogonal => Self::orthogonal_route(start, end, source_node, target_node),
            RoutingType::Curved => Self::curved_route(start, end),
            RoutingType::Auto => Self::auto_route(start, end, source_node, target_node),
        }
    }

    /// Simple straight line routing (default)
    fn straight_route(start: Point, end: Point) -> Vec<[i32; 2]> {
        vec![
            [0, 0],
            [
                (end.0 - start.0).round() as i32,
                (end.1 - start.1).round() as i32,
            ],
        ]
    }

    /// Orthogonal (Manhattan) routing with right angles
    fn orthogonal_route(
        start: Point,
        end: Point,
        source: &NodeData,
        target: &NodeData,
    ) -> Vec<[i32; 2]> {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        // Calculate bounds from node position and size
        let source_min_x = source.x;
        let source_min_y = source.y;
        let source_max_x = source.x + source.width;
        let source_max_y = source.y + source.height;

        let target_min_x = target.x;
        let target_min_y = target.y;
        let target_max_x = target.x + target.width;
        let target_max_y = target.y + target.height;

        // Check if nodes are more horizontally or vertically aligned
        let horizontal_overlap = source_max_x > target_min_x && source_min_x < target_max_x;

        let vertical_overlap = source_max_y > target_min_y && source_min_y < target_max_y;

        let mut points = vec![[0, 0]];

        if horizontal_overlap && !vertical_overlap {
            // Nodes are horizontally aligned, route vertically first
            let mid_y = dy / 2.0;
            points.push([0, mid_y.round() as i32]);
            points.push([dx.round() as i32, mid_y.round() as i32]);
        } else if vertical_overlap && !horizontal_overlap {
            // Nodes are vertically aligned, route horizontally first
            let mid_x = dx / 2.0;
            points.push([mid_x.round() as i32, 0]);
            points.push([mid_x.round() as i32, dy.round() as i32]);
        } else {
            // Default: horizontal first, then vertical
            let _offset = 30.0; // Minimum offset from nodes

            if dx.abs() > dy.abs() {
                // Mainly horizontal connection
                let mid_x = dx / 2.0;
                points.push([mid_x.round() as i32, 0]);
                points.push([mid_x.round() as i32, dy.round() as i32]);
            } else {
                // Mainly vertical connection
                let mid_y = dy / 2.0;
                points.push([0, mid_y.round() as i32]);
                points.push([dx.round() as i32, mid_y.round() as i32]);
            }
        }

        points.push([dx.round() as i32, dy.round() as i32]);
        points
    }

    /// Curved routing using bezier-like control points
    fn curved_route(start: Point, end: Point) -> Vec<[i32; 2]> {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        // Create a simple curved path with one control point
        let control_offset = dx.abs().min(dy.abs()) * 0.3;

        let mut points = vec![[0, 0]];

        if dx.abs() > dy.abs() {
            // Horizontal curve
            let mid_x = dx / 2.0;
            let control_y = if dy > 0.0 {
                control_offset
            } else {
                -control_offset
            };
            points.push([mid_x.round() as i32, control_y.round() as i32]);
        } else {
            // Vertical curve
            let mid_y = dy / 2.0;
            let control_x = if dx > 0.0 {
                control_offset
            } else {
                -control_offset
            };
            points.push([control_x.round() as i32, mid_y.round() as i32]);
        }

        points.push([dx.round() as i32, dy.round() as i32]);
        points
    }

    /// Automatic routing - chooses the best algorithm based on node arrangement
    fn auto_route(
        start: Point,
        end: Point,
        _source: &NodeData,
        _target: &NodeData,
    ) -> Vec<[i32; 2]> {
        // Default to straight line routing for all connections
        // This provides cleaner, more readable diagrams by default
        Self::straight_route(start, end)
    }

    /// Advanced routing with obstacle avoidance using A* pathfinding
    pub fn route_with_avoidance(
        start: Point,
        end: Point,
        source_node: &NodeData,
        target_node: &NodeData,
        obstacles: &[NodeData],
        routing_type: Option<RoutingType>,
    ) -> Vec<[i32; 2]> {
        // If no obstacles or using straight routing, use basic routing
        if obstacles.is_empty() || matches!(routing_type, Some(RoutingType::Straight)) {
            return Self::route_edge(start, end, source_node, target_node, routing_type);
        }

        // Convert obstacles to obstacle structs (excluding source and target nodes)
        let obstacle_rects: Vec<Obstacle> = obstacles
            .iter()
            .filter(|node| node.id != source_node.id && node.id != target_node.id)
            .map(Obstacle::new)
            .collect();

        // Check if direct path is clear
        let start_i = (start.0.round() as i32, start.1.round() as i32);
        let end_i = (end.0.round() as i32, end.1.round() as i32);

        if !Self::path_intersects_obstacles(&obstacle_rects, start_i, end_i) {
            return Self::route_edge(start, end, source_node, target_node, routing_type);
        }

        // Use A* pathfinding to find route around obstacles
        match Self::find_path_astar(start_i, end_i, &obstacle_rects) {
            Some(path) => path.into_iter().map(|p| [p.0, p.1]).collect(),
            None => {
                // Fallback to basic routing if pathfinding fails
                Self::route_edge(start, end, source_node, target_node, routing_type)
            }
        }
    }

    fn path_intersects_obstacles(
        obstacles: &[Obstacle],
        start: (i32, i32),
        end: (i32, i32),
    ) -> bool {
        obstacles
            .iter()
            .any(|obstacle| obstacle.intersects_line(start, end))
    }

    fn find_path_astar(
        start: (i32, i32),
        end: (i32, i32),
        obstacles: &[Obstacle],
    ) -> Option<Vec<(i32, i32)>> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();

        // Initialize start node
        let start_node = AStarNode {
            pos: start,
            g_cost: 0.0,
            h_cost: Self::heuristic(start, end),
            f_cost: Self::heuristic(start, end),
            parent: None,
        };

        open_set.push(start_node);
        g_score.insert(start, 0.0);

        // Maximum iterations to prevent infinite loops
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 1000;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                break;
            }

            if current.pos == end {
                // Reconstruct path
                return Some(Self::reconstruct_path(&came_from, current.pos));
            }

            closed_set.insert(current.pos);

            // Get neighbors (8-directional movement)
            for neighbor_pos in Self::get_neighbors(current.pos) {
                if closed_set.contains(&neighbor_pos) {
                    continue;
                }

                // Check if neighbor is inside an obstacle
                if obstacles
                    .iter()
                    .any(|obs| obs.contains_point(neighbor_pos.0, neighbor_pos.1))
                {
                    continue;
                }

                let tentative_g_score = current.g_cost + Self::distance(current.pos, neighbor_pos);

                if let Some(&existing_g) = g_score.get(&neighbor_pos) {
                    if tentative_g_score >= existing_g {
                        continue;
                    }
                }

                came_from.insert(neighbor_pos, current.pos);
                g_score.insert(neighbor_pos, tentative_g_score);

                let neighbor_node = AStarNode {
                    pos: neighbor_pos,
                    g_cost: tentative_g_score,
                    h_cost: Self::heuristic(neighbor_pos, end),
                    f_cost: tentative_g_score + Self::heuristic(neighbor_pos, end),
                    parent: Some(current.pos),
                };

                open_set.push(neighbor_node);
            }
        }

        None // No path found
    }

    fn heuristic(a: (i32, i32), b: (i32, i32)) -> f64 {
        // Manhattan distance
        ((a.0 - b.0).abs() + (a.1 - b.1).abs()) as f64
    }

    fn distance(a: (i32, i32), b: (i32, i32)) -> f64 {
        // Euclidean distance
        (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f64).sqrt()
    }

    fn get_neighbors(pos: (i32, i32)) -> Vec<(i32, i32)> {
        let grid_size = 20; // Move in 20-pixel increments for efficiency
        vec![
            (pos.0 - grid_size, pos.1),             // Left
            (pos.0 + grid_size, pos.1),             // Right
            (pos.0, pos.1 - grid_size),             // Up
            (pos.0, pos.1 + grid_size),             // Down
            (pos.0 - grid_size, pos.1 - grid_size), // Up-Left
            (pos.0 + grid_size, pos.1 - grid_size), // Up-Right
            (pos.0 - grid_size, pos.1 + grid_size), // Down-Left
            (pos.0 + grid_size, pos.1 + grid_size), // Down-Right
        ]
    }

    fn reconstruct_path(
        came_from: &HashMap<(i32, i32), (i32, i32)>,
        mut current: (i32, i32),
    ) -> Vec<(i32, i32)> {
        let mut path = vec![current];

        while let Some(&parent) = came_from.get(&current) {
            current = parent;
            path.push(current);
        }

        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(x: f64, y: f64, width: f64, height: f64) -> NodeData {
        NodeData {
            id: "test".to_string(),
            label: "test".to_string(),
            attributes: Default::default(),
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn test_straight_route() {
        let start = (0.0, 0.0);
        let end = (100.0, 100.0);
        let points = EdgeRouter::straight_route(start, end);

        assert_eq!(points.len(), 2);
        assert_eq!(points[0], [0, 0]);
        assert_eq!(points[1], [100, 100]);
    }

    #[test]
    fn test_orthogonal_route() {
        let source = create_test_node(0.0, 0.0, 50.0, 50.0);
        let target = create_test_node(100.0, 100.0, 50.0, 50.0);

        let start = (25.0, 25.0);
        let end = (125.0, 125.0);

        let points = EdgeRouter::orthogonal_route(start, end, &source, &target);

        assert!(points.len() >= 3); // Should have at least one intermediate point
        assert_eq!(points[0], [0, 0]);
        assert_eq!(points[points.len() - 1], [100, 100]);
    }

    #[test]
    fn test_curved_route() {
        let start = (0.0, 0.0);
        let end = (100.0, 50.0);
        let points = EdgeRouter::curved_route(start, end);

        assert_eq!(points.len(), 3); // Start, control point, end
        assert_eq!(points[0], [0, 0]);
        assert_eq!(points[2], [100, 50]);
        // Control point should be offset from the straight line
        assert_ne!(points[1][1], 25); // Not on the straight line
    }
}
