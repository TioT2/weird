/// WAT3RS Project
/// `File` map.rs
/// `Description` Map implementation module
/// `Author` TioT2
/// `Last changed` 04.06.2024

use crate::Vec2f;

/// Single edge representation.
pub struct Edge {
    /// Line base point
    pub position: Vec2f,
    /// position -> next position vector
    pub direction: Vec2f,
    /// direction % position cache variable
    pub direction_cross_position: f32,
    /// Line type
    pub ty: EdgeType,
} // struct Edge

/// Edge kind.
pub enum EdgeType {
    /// Portal to another sector
    Portal {
        sector_id: u32,
    },
    /// Portal to subsector
    Subportal {
        subsector_id: u32,
    },
    /// Just wall
    Wall {

    },
} // enum EdgeType

/// Subsector. Subsectors are just parts of sectors that concave sectors are splitted in.
/// Also they require less data and are much more performant than standard sectors in rendering.
pub struct Subsector {
    /// Set of subsector edges
    pub edges: Vec<Edge>,
} // struct Subsector

/// Sector. Sector is unit part of the map.
pub struct Sector {
    /// Set of sector convex subsectors
    pub subsectors: Vec<Subsector>,
    /// Height of subsector floor
    pub floor: f32,
    /// Height of subsector ceiling
    pub ceiling: f32,
} // struct Sector

/// Map representation structure
pub struct Map {
    /// Map sector set
    pub sectors: Vec<Sector>,
} // struct Map

// file map.rs
