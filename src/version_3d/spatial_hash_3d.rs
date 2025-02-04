use std::collections::HashMap;

use macroquad::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CellCoords(i32, i32, i32);

#[derive(Debug)]

pub struct SpatialHash<ID> {
    cell_size: f32,
    grid: HashMap<CellCoords, Vec<ID>>, // Mapping of cell coordinates to object IDs
}

impl<ID: Copy + Eq> SpatialHash<ID> {
    /// Creates a new SpatialHash with the given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    /// Converts a position vector to a cell coordinate
    fn to_cell_coords(&self, position: Vec3) -> CellCoords {
        CellCoords(
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }

    /// Inserts an object ID into the spatial hash
    pub fn insert(&mut self, position: Vec3, id: ID) {
        let cell_coords = self.to_cell_coords(position);
        self.grid
            .entry(cell_coords)
            .or_insert_with(Vec::new)
            .push(id);
    }

    /// Removes an object ID from the spatial hash
    // pub fn remove(&mut self, position: Vec3, id: ID) {
    //     if let Some(cell) = self.grid.get_mut(&self.to_cell_coords(position)) {
    //         if let Some(pos) = cell.iter().position(|&stored_id| stored_id == id) {
    //             cell.remove(pos);
    //         }
    //     }
    // }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    /// Returns a list of object IDs in the specified cell
    // pub fn get_objects_in_cell(&self, position: Vec3) -> Option<&Vec<ID>> {
    //     let cell_coords = self.to_cell_coords(position);
    //     self.grid.get(&cell_coords)
    // }

    /// Returns a list of object IDs within the surrounding cells
    pub fn get_nearby_objects(&self, position: Vec3, id: ID) -> Vec<ID> {
        let center_cell = self.to_cell_coords(position);

        let mut nearby_objects = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell_coords =
                        CellCoords(center_cell.0 + dx, center_cell.1 + dy, center_cell.2 + dz);

                    if let Some(objects) = self.grid.get(&cell_coords) {
                        nearby_objects
                            .extend(objects.iter().copied().filter(|&object_id| object_id != id));
                    }
                }
            }
        }

        nearby_objects
    }
}
