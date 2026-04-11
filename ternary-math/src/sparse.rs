// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file contains trade secrets of Capomastro Holdings Ltd.
// Unauthorized copying, distribution, or use is strictly prohibited.
//
// sparse.rs — Sparse torus cell storage for the Salvi Framework.
// For large tori (cycle length C >> occupied cells), stores only
// non-zero entries. Used by Forma Codex grid engine and compression engine.

use std::collections::HashMap;
use crate::trit_int::TritInt;

// ══════════════════════════════════════════════════════════════
// SPARSE MAP
// ══════════════════════════════════════════════════════════════

/// Sparse map for torus cell data.
///
/// Stores only occupied cells. Coordinates are TritInt values bounded
/// by moduli. Internal key is a TritInt encoded via mixed-radix:
///   key = coord[0] + coord[1]×m₀ + coord[2]×m₀×m₁ + ...
///
/// Zero allocation overhead per insert/get beyond TritInt arithmetic.
/// The heap path (Phase 6) handles keys of any size — no overflow constraint.
///
/// Requires moduli at construction (the torus dimensions).
/// Coordinate validation: each coord[i] must be < moduli[i].
pub struct SparseMap<V> {
    entries: HashMap<TritInt, V>,
    moduli: Vec<TritInt>,
    /// Precomputed mixed-radix strides: strides[i] = product of moduli[0..i]
    strides: Vec<TritInt>,
}

impl<V> SparseMap<V> {
    /// Create a new SparseMap with the given moduli (torus dimensions).
    pub fn new(moduli: Vec<TritInt>) -> Self {
        let mut strides = Vec::with_capacity(moduli.len());
        let mut stride = TritInt::one();
        for m in &moduli {
            strides.push(stride.clone());
            stride = TritInt::mul(&stride, m);
        }
        SparseMap {
            entries: HashMap::new(),
            moduli,
            strides,
        }
    }

    /// Encode a coordinate vector into a single TritInt key via mixed-radix.
    fn encode(&self, coord: &[TritInt]) -> TritInt {
        assert_eq!(coord.len(), self.moduli.len(),
            "coordinate dimension {} != map dimension {}",
            coord.len(), self.moduli.len());
        let mut key = TritInt::zero();
        for (i, c) in coord.iter().enumerate() {
            assert!(*c < self.moduli[i],
                "coord[{}] exceeds modulus", i);
            key = TritInt::add(&key, &TritInt::mul(c, &self.strides[i]));
        }
        key
    }

    /// Decode a TritInt key back into a coordinate vector via successive div_mod.
    fn decode(&self, key: &TritInt) -> Vec<TritInt> {
        let mut remaining = key.clone();
        self.moduli.iter().map(|m| {
            let (q, r) = remaining.div_mod(m);
            remaining = q;
            r
        }).collect()
    }

    /// Insert a value at the given coordinate. Returns the previous value if any.
    pub fn insert(&mut self, coord: &[TritInt], value: V) -> Option<V> {
        let key = self.encode(coord);
        self.entries.insert(key, value)
    }

    /// Get a reference to the value at the given coordinate.
    pub fn get(&self, coord: &[TritInt]) -> Option<&V> {
        let key = self.encode(coord);
        self.entries.get(&key)
    }

    /// Get a mutable reference to the value at the given coordinate.
    pub fn get_mut(&mut self, coord: &[TritInt]) -> Option<&mut V> {
        let key = self.encode(coord);
        self.entries.get_mut(&key)
    }

    /// Remove and return the value at the given coordinate.
    pub fn remove(&mut self, coord: &[TritInt]) -> Option<V> {
        let key = self.encode(coord);
        self.entries.remove(&key)
    }

    /// True if a value exists at the given coordinate.
    pub fn contains(&self, coord: &[TritInt]) -> bool {
        let key = self.encode(coord);
        self.entries.contains_key(&key)
    }

    /// Number of occupied cells.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no cells are occupied.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all occupied cells as (decoded coordinate, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (Vec<TritInt>, &V)> + '_ {
        self.entries.iter().map(|(k, v)| (self.decode(k), v))
    }

    /// Iterate over all occupied coordinates (decoded).
    pub fn coords(&self) -> impl Iterator<Item = Vec<TritInt>> + '_ {
        self.entries.keys().map(|k| self.decode(k))
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of dimensions (number of moduli).
    pub fn dimensions(&self) -> usize {
        self.moduli.len()
    }

    /// The moduli (torus dimension bounds).
    pub fn moduli(&self) -> &[TritInt] {
        &self.moduli
    }
}

// ══════════════════════════════════════════════════════════════
// Z-STACK
// ══════════════════════════════════════════════════════════════

/// Z-stack: layered sparse maps at different depths/resolutions.
///
/// Each layer is a SparseMap with its own moduli. Deeper layers
/// correspond to higher-depth repunit decompositions (larger cycle
/// lengths, finer resolution).
pub struct ZStack<V> {
    layers: Vec<SparseMap<V>>,
}

impl<V> ZStack<V> {
    /// Create an empty Z-stack.
    pub fn new() -> Self {
        ZStack { layers: Vec::new() }
    }

    /// Push a new layer onto the stack.
    pub fn push_layer(&mut self, layer: SparseMap<V>) {
        self.layers.push(layer);
    }

    /// Get a reference to the layer at the given index.
    pub fn layer(&self, index: usize) -> Option<&SparseMap<V>> {
        self.layers.get(index)
    }

    /// Get a mutable reference to the layer at the given index.
    pub fn layer_mut(&mut self, index: usize) -> Option<&mut SparseMap<V>> {
        self.layers.get_mut(index)
    }

    /// Number of layers.
    pub fn depth(&self) -> usize {
        self.layers.len()
    }
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_insert_get() {
        let m = vec![TritInt::from_u64(10), TritInt::from_u64(10)];
        let mut map: SparseMap<u64> = SparseMap::new(m);
        map.insert(&[TritInt::zero(), TritInt::zero()], 42);
        map.insert(&[TritInt::from_u64(3), TritInt::from_u64(7)], 99);
        assert_eq!(map.get(&[TritInt::zero(), TritInt::zero()]), Some(&42));
        assert_eq!(map.get(&[TritInt::from_u64(3), TritInt::from_u64(7)]), Some(&99));
        assert_eq!(map.get(&[TritInt::one(), TritInt::zero()]), None);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_sparse_remove() {
        let m = vec![TritInt::from_u64(10), TritInt::from_u64(10)];
        let mut map: SparseMap<u64> = SparseMap::new(m);
        map.insert(&[TritInt::zero(), TritInt::zero()], 42);
        assert_eq!(map.remove(&[TritInt::zero(), TritInt::zero()]), Some(42));
        assert!(map.is_empty());
    }

    #[test]
    fn test_sparse_overwrite() {
        let m = vec![TritInt::from_u64(10)];
        let mut map: SparseMap<u64> = SparseMap::new(m);
        map.insert(&[TritInt::from_u64(5)], 10);
        let old = map.insert(&[TritInt::from_u64(5)], 20);
        assert_eq!(old, Some(10));
        assert_eq!(map.get(&[TritInt::from_u64(5)]), Some(&20));
    }

    #[test]
    fn test_zstack_layers() {
        let mut stack: ZStack<u8> = ZStack::new();
        let mut layer0 = SparseMap::new(
            vec![TritInt::from_u64(10), TritInt::from_u64(10)]);
        layer0.insert(&[TritInt::zero(), TritInt::zero()], 1);
        stack.push_layer(layer0);
        let mut layer1 = SparseMap::new(
            vec![TritInt::from_u64(10), TritInt::from_u64(10), TritInt::from_u64(10)]);
        layer1.insert(&[TritInt::zero(), TritInt::zero(), TritInt::zero()], 2);
        stack.push_layer(layer1);
        assert_eq!(stack.depth(), 2);
        assert_eq!(
            stack.layer(0).unwrap().get(&[TritInt::zero(), TritInt::zero()]),
            Some(&1));
        assert_eq!(
            stack.layer(1).unwrap().get(&[TritInt::zero(), TritInt::zero(), TritInt::zero()]),
            Some(&2));
    }

    #[test]
    fn test_sparse_iter() {
        let m = vec![TritInt::from_u64(10), TritInt::from_u64(10)];
        let mut map: SparseMap<u64> = SparseMap::new(m);
        map.insert(&[TritInt::one(), TritInt::from_u64(2)], 10);
        map.insert(&[TritInt::from_u64(3), TritInt::from_u64(4)], 20);
        let collected: Vec<_> = map.iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    #[should_panic(expected = "coordinate dimension")]
    fn test_sparse_dimension_mismatch() {
        let mut map: SparseMap<u64> = SparseMap::new(
            vec![TritInt::from_u64(10), TritInt::from_u64(10)]);
        map.insert(
            &[TritInt::one(), TritInt::from_u64(2), TritInt::from_u64(3)],
            42);
    }

    #[test]
    #[should_panic(expected = "exceeds modulus")]
    fn test_sparse_coord_out_of_bounds() {
        let mut map: SparseMap<u64> = SparseMap::new(
            vec![TritInt::from_u64(5), TritInt::from_u64(5)]);
        map.insert(&[TritInt::from_u64(7), TritInt::from_u64(2)], 42);
    }
}
