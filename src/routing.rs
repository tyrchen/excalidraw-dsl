// src/routing.rs
use crate::ast::RoutingType;
use crate::igr::NodeData;

/// Represents a point in 2D space
pub type Point = (f64, f64);

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
    fn auto_route(start: Point, end: Point, source: &NodeData, target: &NodeData) -> Vec<[i32; 2]> {
        let dx = (end.0 - start.0).abs();
        let dy = (end.1 - start.1).abs();

        // Use orthogonal routing for connections that are mostly aligned
        if dx < 50.0 || dy < 50.0 {
            Self::orthogonal_route(start, end, source, target)
        } else {
            // Use straight line for diagonal connections
            Self::straight_route(start, end)
        }
    }

    /// Advanced routing with obstacle avoidance
    pub fn route_with_avoidance(
        start: Point,
        end: Point,
        source_node: &NodeData,
        target_node: &NodeData,
        _obstacles: &[NodeData],
        routing_type: Option<RoutingType>,
    ) -> Vec<[i32; 2]> {
        // For now, just use basic routing
        // TODO: Implement A* or similar pathfinding algorithm for obstacle avoidance
        Self::route_edge(start, end, source_node, target_node, routing_type)
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
