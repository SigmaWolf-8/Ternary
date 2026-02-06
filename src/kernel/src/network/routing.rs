use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::{NetworkError, NetworkResult, NodeId};
use super::torus::{TorusCoordinate, TorusTopology};

#[derive(Debug, Clone)]
pub struct RouteHop {
    pub node_id: NodeId,
    pub coordinate: TorusCoordinate,
    pub cost: u64,
    pub hop_number: u16,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub hops: Vec<RouteHop>,
    pub total_cost: u64,
    pub dimension_path: Vec<usize>,
}

pub struct RoutingTable {
    routes_cache: BTreeMap<(u64, u64), Route>,
    max_hops: u16,
}

impl RoutingTable {
    pub fn new(max_hops: u16) -> Self {
        Self {
            routes_cache: BTreeMap::new(),
            max_hops,
        }
    }

    pub fn find_route(&self, topology: &TorusTopology, from: NodeId, to: NodeId) -> NetworkResult<Route> {
        let src_coord = topology.get_node(from).ok_or(NetworkError::NodeNotFound)?;
        let dst_coord = topology.get_node(to).ok_or(NetworkError::NodeNotFound)?;

        if src_coord == dst_coord {
            return Ok(Route {
                hops: Vec::new(),
                total_cost: 0,
                dimension_path: Vec::new(),
            });
        }

        let side_lengths = topology.side_lengths();
        let dims = topology.dimensions();
        let mut current = src_coord.clone();
        let mut hops = Vec::new();
        let mut dimension_path = Vec::new();
        let mut total_cost = 0u64;
        let mut hop_number = 0u16;

        while current != *dst_coord {
            if hop_number >= self.max_hops {
                return Err(NetworkError::MaxHopsExceeded);
            }

            let mut best_dim = 0usize;
            let mut best_reduction: u64 = 0;

            for d in 0..dims {
                let cv = current.coords[d] as i32;
                let dv = dst_coord.coords[d] as i32;
                let l = side_lengths[d] as i32;
                let diff = (cv - dv).unsigned_abs();
                let wrap_dist = core::cmp::min(diff, l as u32 - diff);
                if wrap_dist == 0 {
                    continue;
                }
                let weight = topology.get_coefficient(d)
                    .map(|c| c.effective_weight() as u64)
                    .unwrap_or(1000);
                let reduction = wrap_dist as u64 * weight;
                if reduction > best_reduction {
                    best_reduction = reduction;
                    best_dim = d;
                }
            }

            let cv = current.coords[best_dim] as i32;
            let dv = dst_coord.coords[best_dim] as i32;
            let l = side_lengths[best_dim] as i32;
            let forward = ((dv - cv).rem_euclid(l)) as u32;
            let backward = ((cv - dv).rem_euclid(l)) as u32;

            if forward <= backward {
                current.coords[best_dim] = ((cv + 1).rem_euclid(l)) as u16;
            } else {
                current.coords[best_dim] = ((cv - 1).rem_euclid(l)) as u16;
            }

            let weight = topology.get_coefficient(best_dim)
                .map(|c| c.effective_weight() as u64)
                .unwrap_or(1000);
            total_cost += weight;
            dimension_path.push(best_dim);

            hops.push(RouteHop {
                node_id: to,
                coordinate: current.clone(),
                cost: weight,
                hop_number,
            });

            hop_number += 1;
        }

        Ok(Route {
            hops,
            total_cost,
            dimension_path,
        })
    }

    pub fn cache_route(&mut self, from: NodeId, to: NodeId, route: Route) {
        self.routes_cache.insert((from.0, to.0), route);
    }

    pub fn get_cached(&self, from: NodeId, to: NodeId) -> Option<&Route> {
        self.routes_cache.get(&(from.0, to.0))
    }

    pub fn clear_cache(&mut self) {
        self.routes_cache.clear();
    }

    pub fn invalidate_node(&mut self, node: NodeId) {
        let nid = node.0;
        self.routes_cache.retain(|&(src, dst), _| src != nid && dst != nid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use super::super::torus::DimensionType;

    fn make_1d_topology() -> (TorusTopology, NodeId, NodeId) {
        let mut topo = TorusTopology::new(vec![5]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![2])).unwrap();
        (topo, a, b)
    }

    #[test]
    fn test_route_simple_1d() {
        let (topo, a, b) = make_1d_topology();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 2);
        assert_eq!(route.dimension_path.len(), 2);
    }

    #[test]
    fn test_route_2d_no_wrap() {
        let mut topo = TorusTopology::new(vec![5, 5]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![2, 1])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 3);
    }

    #[test]
    fn test_route_2d_with_wrap() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![2, 2])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 2);
    }

    #[test]
    fn test_route_same_node() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![1, 1])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![1, 1])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 0);
        assert_eq!(route.total_cost, 0);
    }

    #[test]
    fn test_route_max_hops_exceeded() {
        let mut topo = TorusTopology::new(vec![100]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![50])).unwrap();
        let rt = RoutingTable::new(5);
        let result = rt.find_route(&topo, a, b);
        assert!(result.is_err());
    }

    #[test]
    fn test_route_cache() {
        let (topo, a, b) = make_1d_topology();
        let mut rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        let cost = route.total_cost;
        rt.cache_route(a, b, route);
        let cached = rt.get_cached(a, b).unwrap();
        assert_eq!(cached.total_cost, cost);
    }

    #[test]
    fn test_cache_invalidation() {
        let (topo, a, b) = make_1d_topology();
        let mut rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        rt.cache_route(a, b, route);
        assert!(rt.get_cached(a, b).is_some());
        rt.invalidate_node(a);
        assert!(rt.get_cached(a, b).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let (topo, a, b) = make_1d_topology();
        let mut rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        rt.cache_route(a, b, route);
        rt.clear_cache();
        assert!(rt.get_cached(a, b).is_none());
    }

    #[test]
    fn test_route_weighted() {
        let mut topo = TorusTopology::new(vec![5, 5]).unwrap();
        topo.set_coefficient(0, 500, DimensionType::Spatial).unwrap();
        topo.set_coefficient(1, 2000, DimensionType::Security).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![1, 1])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 2);
        assert!(route.total_cost > 0);
    }

    #[test]
    fn test_geodesic_prefers_high_weight_dimensions() {
        let mut topo = TorusTopology::new(vec![5, 5]).unwrap();
        topo.set_coefficient(0, 100, DimensionType::Spatial).unwrap();
        topo.set_coefficient(1, 5000, DimensionType::Security).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![1, 1])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.dimension_path[0], 1);
    }

    #[test]
    fn test_route_7d() {
        let mut topo = super::super::torus::torus_7d();
        let a = topo.add_node(TorusCoordinate::new(vec![0, 0, 0, 0, 0, 0, 0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![1, 1, 1, 1, 1, 1, 1])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 7);
    }

    #[test]
    fn test_route_13d() {
        let mut topo = super::super::torus::torus_13d();
        let a = topo.add_node(TorusCoordinate::new(vec![0; 13])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![1; 13])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 13);
    }

    #[test]
    fn test_route_no_source_node() {
        let topo = TorusTopology::new(vec![3]).unwrap();
        let rt = RoutingTable::new(256);
        let result = rt.find_route(&topo, NodeId(0), NodeId(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_route_no_dest_node() {
        let mut topo = TorusTopology::new(vec![3]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0])).unwrap();
        let rt = RoutingTable::new(256);
        let result = rt.find_route(&topo, a, NodeId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_route_dimension_path_recorded() {
        let mut topo = TorusTopology::new(vec![5, 5]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![2, 0])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert!(route.dimension_path.iter().all(|&d| d == 0));
    }

    #[test]
    fn test_route_total_cost_consistency() {
        let mut topo = TorusTopology::new(vec![5]).unwrap();
        topo.set_coefficient(0, 1000, DimensionType::Spatial).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![2])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        let sum: u64 = route.hops.iter().map(|h| h.cost).sum();
        assert_eq!(route.total_cost, sum);
    }

    #[test]
    fn test_cache_invalidate_preserves_unrelated() {
        let mut topo = TorusTopology::new(vec![5]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![1])).unwrap();
        let c = topo.add_node(TorusCoordinate::new(vec![2])).unwrap();
        let mut rt = RoutingTable::new(256);
        let r1 = rt.find_route(&topo, a, b).unwrap();
        let r2 = rt.find_route(&topo, b, c).unwrap();
        rt.cache_route(a, b, r1);
        rt.cache_route(b, c, r2);
        rt.invalidate_node(a);
        assert!(rt.get_cached(a, b).is_none());
        assert!(rt.get_cached(b, c).is_some());
    }

    #[test]
    fn test_routing_table_new() {
        let rt = RoutingTable::new(128);
        assert!(rt.get_cached(NodeId(0), NodeId(1)).is_none());
    }

    #[test]
    fn test_route_wrap_direction() {
        let mut topo = TorusTopology::new(vec![5]).unwrap();
        let a = topo.add_node(TorusCoordinate::new(vec![0])).unwrap();
        let b = topo.add_node(TorusCoordinate::new(vec![4])).unwrap();
        let rt = RoutingTable::new(256);
        let route = rt.find_route(&topo, a, b).unwrap();
        assert_eq!(route.hops.len(), 1);
    }
}
