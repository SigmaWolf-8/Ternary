use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use super::{NetworkError, NetworkResult, NodeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorusCoordinate {
    pub coords: Vec<u16>,
}

impl TorusCoordinate {
    pub fn new(coords: Vec<u16>) -> Self {
        Self { coords }
    }

    pub fn dimension(&self) -> usize {
        self.coords.len()
    }

    pub fn get(&self, dim: usize) -> Option<u16> {
        self.coords.get(dim).copied()
    }

    pub fn set(&mut self, dim: usize, val: u16) -> NetworkResult<()> {
        if dim >= self.coords.len() {
            return Err(NetworkError::InvalidDimension);
        }
        self.coords[dim] = val;
        Ok(())
    }

    pub fn distance_to(&self, other: &Self, side_lengths: &[u16]) -> u32 {
        let mut total = 0u32;
        for i in 0..self.coords.len() {
            let a = self.coords[i] as i32;
            let b = other.coords[i] as i32;
            let l = side_lengths[i] as i32;
            let diff = (a - b).unsigned_abs();
            let wrap = l as u32 - diff;
            total += core::cmp::min(diff, wrap);
        }
        total
    }

    pub fn neighbors(&self, side_lengths: &[u16]) -> Vec<TorusCoordinate> {
        let mut result = Vec::new();
        for d in 0..self.coords.len() {
            let l = side_lengths[d];
            let mut plus = self.coords.clone();
            plus[d] = (plus[d] + 1) % l;
            result.push(TorusCoordinate::new(plus));

            let mut minus = self.coords.clone();
            minus[d] = (minus[d] + l - 1) % l;
            result.push(TorusCoordinate::new(minus));
        }
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionType {
    Spatial,
    Temporal,
    Phase,
    Security,
    Ternary,
    Frequency,
    Polarization,
}

#[derive(Debug, Clone)]
pub struct TorsionCoefficient {
    pub dimension: usize,
    pub weight: u32,
    pub dim_type: DimensionType,
}

impl TorsionCoefficient {
    pub fn new(dimension: usize, weight: u32, dim_type: DimensionType) -> Self {
        Self { dimension, weight, dim_type }
    }

    pub fn effective_weight(&self) -> u32 {
        self.weight
    }
}

#[derive(Debug)]
pub struct TorusTopology {
    dimensions: usize,
    side_lengths: Vec<u16>,
    coefficients: Vec<TorsionCoefficient>,
    nodes: BTreeMap<NodeId, TorusCoordinate>,
    next_node_id: u64,
}

impl TorusTopology {
    pub fn new(side_lengths: Vec<u16>) -> NetworkResult<Self> {
        if side_lengths.is_empty() {
            return Err(NetworkError::InvalidDimension);
        }
        let dimensions = side_lengths.len();
        let mut coefficients = Vec::with_capacity(dimensions);
        for i in 0..dimensions {
            coefficients.push(TorsionCoefficient::new(i, 1000, DimensionType::Spatial));
        }
        Ok(Self {
            dimensions,
            side_lengths,
            coefficients,
            nodes: BTreeMap::new(),
            next_node_id: 0,
        })
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub fn side_lengths(&self) -> &[u16] {
        &self.side_lengths
    }

    pub fn total_positions(&self) -> u64 {
        self.side_lengths.iter().fold(1u64, |acc, &l| acc * l as u64)
    }

    pub fn add_node(&mut self, coord: TorusCoordinate) -> NetworkResult<NodeId> {
        self.validate_coordinate(&coord)?;
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.insert(id, coord);
        Ok(id)
    }

    pub fn remove_node(&mut self, id: NodeId) -> NetworkResult<()> {
        if self.nodes.remove(&id).is_none() {
            return Err(NetworkError::NodeNotFound);
        }
        Ok(())
    }

    pub fn get_node(&self, id: NodeId) -> Option<&TorusCoordinate> {
        self.nodes.get(&id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn set_coefficient(&mut self, dim: usize, weight: u32, dim_type: DimensionType) -> NetworkResult<()> {
        if dim >= self.dimensions {
            return Err(NetworkError::InvalidDimension);
        }
        self.coefficients[dim] = TorsionCoefficient::new(dim, weight, dim_type);
        Ok(())
    }

    pub fn get_coefficient(&self, dim: usize) -> Option<&TorsionCoefficient> {
        self.coefficients.get(dim)
    }

    pub fn weighted_distance(&self, a: &TorusCoordinate, b: &TorusCoordinate) -> u64 {
        let mut total = 0u64;
        for i in 0..self.dimensions {
            let av = a.coords[i] as i32;
            let bv = b.coords[i] as i32;
            let l = self.side_lengths[i] as i32;
            let diff = (av - bv).unsigned_abs();
            let wrap = l as u32 - diff;
            let dist = core::cmp::min(diff, wrap) as u64;
            let weight = self.coefficients[i].effective_weight() as u64;
            total += dist * weight;
        }
        total
    }

    pub fn neighbors_of(&self, id: NodeId) -> NetworkResult<Vec<(NodeId, u32)>> {
        let coord = self.nodes.get(&id).ok_or(NetworkError::NodeNotFound)?;
        let neighbor_coords = coord.neighbors(&self.side_lengths);
        let mut result = Vec::new();
        for nc in &neighbor_coords {
            for (&nid, ncoord) in &self.nodes {
                if ncoord == nc {
                    let dist = coord.distance_to(ncoord, &self.side_lengths);
                    result.push((nid, dist));
                }
            }
        }
        Ok(result)
    }

    pub fn validate_coordinate(&self, coord: &TorusCoordinate) -> NetworkResult<()> {
        if coord.dimension() != self.dimensions {
            return Err(NetworkError::InvalidDimension);
        }
        for i in 0..self.dimensions {
            if coord.coords[i] >= self.side_lengths[i] {
                return Err(NetworkError::InvalidCoordinate);
            }
        }
        Ok(())
    }

    pub fn wrap_coordinate(&self, coord: &mut TorusCoordinate) {
        for i in 0..core::cmp::min(coord.coords.len(), self.dimensions) {
            coord.coords[i] = coord.coords[i] % self.side_lengths[i];
        }
    }
}

pub fn torus_7d() -> TorusTopology {
    let side_lengths = vec![3u16; 7];
    let mut topo = TorusTopology::new(side_lengths).unwrap();
    let types = [
        DimensionType::Spatial, DimensionType::Spatial, DimensionType::Spatial,
        DimensionType::Temporal, DimensionType::Phase, DimensionType::Security,
        DimensionType::Ternary,
    ];
    let weights: [u32; 7] = [1000, 1000, 1000, 800, 1200, 1500, 900];
    for i in 0..7 {
        topo.set_coefficient(i, weights[i], types[i]).unwrap();
    }
    topo
}

pub fn torus_10d() -> TorusTopology {
    let side_lengths = vec![3u16; 10];
    let mut topo = TorusTopology::new(side_lengths).unwrap();
    let types = [
        DimensionType::Spatial, DimensionType::Spatial, DimensionType::Spatial,
        DimensionType::Temporal, DimensionType::Phase, DimensionType::Security,
        DimensionType::Ternary, DimensionType::Frequency, DimensionType::Polarization,
        DimensionType::Spatial,
    ];
    let weights: [u32; 10] = [1000, 1000, 1000, 800, 1200, 1500, 900, 700, 600, 1000];
    for i in 0..10 {
        topo.set_coefficient(i, weights[i], types[i]).unwrap();
    }
    topo
}

pub fn torus_13d() -> TorusTopology {
    let side_lengths = vec![3u16; 13];
    let mut topo = TorusTopology::new(side_lengths).unwrap();
    let types = [
        DimensionType::Spatial, DimensionType::Spatial, DimensionType::Spatial,
        DimensionType::Temporal, DimensionType::Phase, DimensionType::Security,
        DimensionType::Ternary, DimensionType::Frequency, DimensionType::Polarization,
        DimensionType::Spatial, DimensionType::Phase, DimensionType::Security,
        DimensionType::Temporal,
    ];
    let weights: [u32; 13] = [1000, 1000, 1000, 800, 1200, 1500, 900, 700, 600, 1000, 1100, 1400, 850];
    for i in 0..13 {
        topo.set_coefficient(i, weights[i], types[i]).unwrap();
    }
    topo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_creation() {
        let coord = TorusCoordinate::new(vec![0, 1, 2]);
        assert_eq!(coord.dimension(), 3);
        assert_eq!(coord.get(0), Some(0));
        assert_eq!(coord.get(1), Some(1));
        assert_eq!(coord.get(2), Some(2));
        assert_eq!(coord.get(3), None);
    }

    #[test]
    fn test_coordinate_distance_no_wrap() {
        let a = TorusCoordinate::new(vec![0, 0]);
        let b = TorusCoordinate::new(vec![1, 1]);
        let side_lengths = [5u16, 5];
        assert_eq!(a.distance_to(&b, &side_lengths), 2);
    }

    #[test]
    fn test_coordinate_distance_with_wrap() {
        let a = TorusCoordinate::new(vec![0]);
        let b = TorusCoordinate::new(vec![2]);
        let side_lengths = [3u16];
        assert_eq!(a.distance_to(&b, &side_lengths), 1);
    }

    #[test]
    fn test_coordinate_distance_wrap_2d() {
        let a = TorusCoordinate::new(vec![0, 0]);
        let b = TorusCoordinate::new(vec![2, 2]);
        let side_lengths = [3u16, 3];
        assert_eq!(a.distance_to(&b, &side_lengths), 2);
    }

    #[test]
    fn test_coordinate_neighbors() {
        let coord = TorusCoordinate::new(vec![1, 1]);
        let side_lengths = [3u16, 3];
        let neighbors = coord.neighbors(&side_lengths);
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&TorusCoordinate::new(vec![2, 1])));
        assert!(neighbors.contains(&TorusCoordinate::new(vec![0, 1])));
        assert!(neighbors.contains(&TorusCoordinate::new(vec![1, 2])));
        assert!(neighbors.contains(&TorusCoordinate::new(vec![1, 0])));
    }

    #[test]
    fn test_coordinate_neighbors_wrapping() {
        let coord = TorusCoordinate::new(vec![0, 0]);
        let side_lengths = [3u16, 3];
        let neighbors = coord.neighbors(&side_lengths);
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&TorusCoordinate::new(vec![1, 0])));
        assert!(neighbors.contains(&TorusCoordinate::new(vec![2, 0])));
        assert!(neighbors.contains(&TorusCoordinate::new(vec![0, 1])));
        assert!(neighbors.contains(&TorusCoordinate::new(vec![0, 2])));
    }

    #[test]
    fn test_coordinate_set() {
        let mut coord = TorusCoordinate::new(vec![0, 0, 0]);
        assert!(coord.set(1, 5).is_ok());
        assert_eq!(coord.get(1), Some(5));
        assert!(coord.set(10, 5).is_err());
    }

    #[test]
    fn test_torus_creation() {
        let topo = TorusTopology::new(vec![3, 3, 3]).unwrap();
        assert_eq!(topo.dimensions(), 3);
    }

    #[test]
    fn test_torus_creation_empty() {
        let result = TorusTopology::new(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_torus_add_node() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let id = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        assert_eq!(id, NodeId(0));
        assert_eq!(topo.node_count(), 1);
    }

    #[test]
    fn test_torus_remove_node() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let id = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        assert!(topo.remove_node(id).is_ok());
        assert_eq!(topo.node_count(), 0);
    }

    #[test]
    fn test_torus_remove_nonexistent() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        assert!(topo.remove_node(NodeId(999)).is_err());
    }

    #[test]
    fn test_torus_invalid_coordinate() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let result = topo.add_node(TorusCoordinate::new(vec![0, 0, 0]));
        assert!(result.is_err());
    }

    #[test]
    fn test_torus_coordinate_bounds() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let result = topo.add_node(TorusCoordinate::new(vec![3, 0]));
        assert!(result.is_err());
    }

    #[test]
    fn test_torus_7d_preset() {
        let topo = torus_7d();
        assert_eq!(topo.dimensions(), 7);
        assert_eq!(topo.get_coefficient(0).unwrap().dim_type, DimensionType::Spatial);
        assert_eq!(topo.get_coefficient(3).unwrap().dim_type, DimensionType::Temporal);
        assert_eq!(topo.get_coefficient(4).unwrap().dim_type, DimensionType::Phase);
        assert_eq!(topo.get_coefficient(5).unwrap().dim_type, DimensionType::Security);
        assert_eq!(topo.get_coefficient(6).unwrap().dim_type, DimensionType::Ternary);
    }

    #[test]
    fn test_torus_10d_preset() {
        let topo = torus_10d();
        assert_eq!(topo.dimensions(), 10);
        assert_eq!(topo.get_coefficient(7).unwrap().dim_type, DimensionType::Frequency);
        assert_eq!(topo.get_coefficient(8).unwrap().dim_type, DimensionType::Polarization);
        assert_eq!(topo.get_coefficient(9).unwrap().dim_type, DimensionType::Spatial);
    }

    #[test]
    fn test_torus_13d_preset() {
        let topo = torus_13d();
        assert_eq!(topo.dimensions(), 13);
        assert_eq!(topo.get_coefficient(10).unwrap().dim_type, DimensionType::Phase);
        assert_eq!(topo.get_coefficient(11).unwrap().dim_type, DimensionType::Security);
        assert_eq!(topo.get_coefficient(12).unwrap().dim_type, DimensionType::Temporal);
    }

    #[test]
    fn test_7d_total_positions() {
        let topo = torus_7d();
        assert_eq!(topo.total_positions(), 2187);
    }

    #[test]
    fn test_10d_total_positions() {
        let topo = torus_10d();
        assert_eq!(topo.total_positions(), 59049);
    }

    #[test]
    fn test_13d_total_positions() {
        let topo = torus_13d();
        assert_eq!(topo.total_positions(), 1594323);
    }

    #[test]
    fn test_torus_total_positions_custom() {
        let topo = TorusTopology::new(vec![2, 3, 4]).unwrap();
        assert_eq!(topo.total_positions(), 24);
    }

    #[test]
    fn test_torsion_coefficients() {
        let coeff = TorsionCoefficient::new(0, 1500, DimensionType::Security);
        assert_eq!(coeff.dimension, 0);
        assert_eq!(coeff.weight, 1500);
        assert_eq!(coeff.dim_type, DimensionType::Security);
        assert_eq!(coeff.effective_weight(), 1500);
    }

    #[test]
    fn test_torsion_coefficient_modification() {
        let mut topo = TorusTopology::new(vec![3, 3, 3]).unwrap();
        topo.set_coefficient(1, 2000, DimensionType::Phase).unwrap();
        let coeff = topo.get_coefficient(1).unwrap();
        assert_eq!(coeff.weight, 2000);
        assert_eq!(coeff.dim_type, DimensionType::Phase);
    }

    #[test]
    fn test_torsion_coefficient_invalid_dim() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        assert!(topo.set_coefficient(5, 1000, DimensionType::Spatial).is_err());
    }

    #[test]
    fn test_weighted_distance() {
        let mut topo = TorusTopology::new(vec![5, 5]).unwrap();
        topo.set_coefficient(0, 1000, DimensionType::Spatial).unwrap();
        topo.set_coefficient(1, 2000, DimensionType::Security).unwrap();
        let a = TorusCoordinate::new(vec![0, 0]);
        let b = TorusCoordinate::new(vec![1, 1]);
        assert_eq!(topo.weighted_distance(&a, &b), 1 * 1000 + 1 * 2000);
    }

    #[test]
    fn test_weighted_distance_wrap() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        topo.set_coefficient(0, 1000, DimensionType::Spatial).unwrap();
        topo.set_coefficient(1, 1000, DimensionType::Spatial).unwrap();
        let a = TorusCoordinate::new(vec![0, 0]);
        let b = TorusCoordinate::new(vec![2, 2]);
        assert_eq!(topo.weighted_distance(&a, &b), 2000);
    }

    #[test]
    fn test_torus_neighbors_of() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let id0 = topo.add_node(TorusCoordinate::new(vec![1, 1])).unwrap();
        let id1 = topo.add_node(TorusCoordinate::new(vec![2, 1])).unwrap();
        let _id2 = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let neighbors = topo.neighbors_of(id0).unwrap();
        assert!(neighbors.iter().any(|(nid, _)| *nid == id1));
    }

    #[test]
    fn test_torus_neighbors_of_nonexistent() {
        let topo = TorusTopology::new(vec![3, 3]).unwrap();
        assert!(topo.neighbors_of(NodeId(999)).is_err());
    }

    #[test]
    fn test_coordinate_wrapping() {
        let topo = TorusTopology::new(vec![3, 5]).unwrap();
        let mut coord = TorusCoordinate::new(vec![5, 7]);
        topo.wrap_coordinate(&mut coord);
        assert_eq!(coord.get(0), Some(2));
        assert_eq!(coord.get(1), Some(2));
    }

    #[test]
    fn test_torus_node_count() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        assert_eq!(topo.node_count(), 0);
        topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        assert_eq!(topo.node_count(), 1);
        topo.add_node(TorusCoordinate::new(vec![1, 1])).unwrap();
        assert_eq!(topo.node_count(), 2);
    }

    #[test]
    fn test_torus_duplicate_node() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let id1 = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        let id2 = topo.add_node(TorusCoordinate::new(vec![0, 0])).unwrap();
        assert_ne!(id1, id2);
        assert_eq!(topo.node_count(), 2);
    }

    #[test]
    fn test_torus_get_node() {
        let mut topo = TorusTopology::new(vec![3, 3]).unwrap();
        let id = topo.add_node(TorusCoordinate::new(vec![1, 2])).unwrap();
        let coord = topo.get_node(id).unwrap();
        assert_eq!(coord.coords, vec![1, 2]);
    }

    #[test]
    fn test_torus_get_node_nonexistent() {
        let topo = TorusTopology::new(vec![3, 3]).unwrap();
        assert!(topo.get_node(NodeId(0)).is_none());
    }

    #[test]
    fn test_dimension_types() {
        assert_ne!(DimensionType::Spatial, DimensionType::Temporal);
        assert_ne!(DimensionType::Phase, DimensionType::Security);
        assert_ne!(DimensionType::Ternary, DimensionType::Frequency);
        assert_ne!(DimensionType::Frequency, DimensionType::Polarization);
    }

    #[test]
    fn test_validate_coordinate_ok() {
        let topo = TorusTopology::new(vec![3, 3, 3]).unwrap();
        let coord = TorusCoordinate::new(vec![2, 2, 2]);
        assert!(topo.validate_coordinate(&coord).is_ok());
    }

    #[test]
    fn test_validate_coordinate_wrong_dims() {
        let topo = TorusTopology::new(vec![3, 3]).unwrap();
        let coord = TorusCoordinate::new(vec![0]);
        assert!(topo.validate_coordinate(&coord).is_err());
    }

    #[test]
    fn test_validate_coordinate_out_of_bounds() {
        let topo = TorusTopology::new(vec![3, 3]).unwrap();
        let coord = TorusCoordinate::new(vec![0, 3]);
        assert!(topo.validate_coordinate(&coord).is_err());
    }

    #[test]
    fn test_7d_preset_weights() {
        let topo = torus_7d();
        let expected = [1000, 1000, 1000, 800, 1200, 1500, 900];
        for i in 0..7 {
            assert_eq!(topo.get_coefficient(i).unwrap().weight, expected[i]);
        }
    }

    #[test]
    fn test_10d_preset_weights() {
        let topo = torus_10d();
        let expected = [1000, 1000, 1000, 800, 1200, 1500, 900, 700, 600, 1000];
        for i in 0..10 {
            assert_eq!(topo.get_coefficient(i).unwrap().weight, expected[i]);
        }
    }

    #[test]
    fn test_13d_preset_weights() {
        let topo = torus_13d();
        let expected = [1000, 1000, 1000, 800, 1200, 1500, 900, 700, 600, 1000, 1100, 1400, 850];
        for i in 0..13 {
            assert_eq!(topo.get_coefficient(i).unwrap().weight, expected[i]);
        }
    }

    #[test]
    fn test_side_lengths_accessor() {
        let topo = TorusTopology::new(vec![3, 5, 7]).unwrap();
        assert_eq!(topo.side_lengths(), &[3, 5, 7]);
    }

    #[test]
    fn test_coordinate_distance_same() {
        let a = TorusCoordinate::new(vec![1, 1, 1]);
        let side_lengths = [3u16, 3, 3];
        assert_eq!(a.distance_to(&a, &side_lengths), 0);
    }
}
