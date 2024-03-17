
use std::collections::HashSet;
use crate::util::unordered_pair::UnorderedPair;
use crate::math::*;

/// Sector edge representation structure
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EdgeType {
    /// Wall
    Wall,
    /// Portal to some sector
    Portal(SectorId),
} // enum Edge

#[derive(Copy, Clone, Debug)]
pub struct Edge {
    pub p0: Vec2f,
    pub p1: Vec2f,
    pub d: Vec2f,
    pub d_cross_p0: f32,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wall => f.write_str("Wall"),
            Self::Portal(portal) => f.write_fmt(format_args!("Portal({})", portal.as_u32()))
        }
    } // fn fmt
} // impl std::fmt::Display for Edge

/// Sector representation structure
pub struct Sector {
    pub points: Vec<Vec2f>,
    pub edge_types: Vec<EdgeType>,
    pub edges: Vec<Edge>,
    pub floor: f32,
    pub ceiling: f32,
} // struct Sector

impl Sector {
    /// Sector with loop of walls representation structure
    pub fn wall_loop(points: &[Vec2f]) -> Self {
        let mut sector = Self {
            points: points.into(),
            edge_types: {
                let mut edges = Vec::<EdgeType>::with_capacity(points.len());
                edges.resize(points.len(), EdgeType::Wall);
                edges
            },
            edges: Vec::new(),
            floor: 0.0,
            ceiling: 1.0,
        };
        sector.edges = sector.build_edges();
        sector
    } // fn wall_loop

    pub fn build_edges(&self) -> Vec<Edge> {
        let mut edge_lines = Vec::<Edge>::with_capacity(self.points.len());

        for (left, right) in self.points.iter().zip({
            let pit = self.points.iter();
            let mut pit = pit.cycle();
            pit.next();
            pit
        }) {
            let d = Vec2f {
                x: right.x - left.x,
                y: right.y - left.y,
            };

            edge_lines.push(Edge {
                p0: *left,
                p1: *right,
                d_cross_p0: d.x * left.y - d.y * left.x,
                d,
            });
        }

        edge_lines
    }

    /// Check for point being located in sector
    pub fn contains(&self, point: Vec2f) -> bool {
        let mut sign_collector: u8 = 0;

        for line in &self.edges {
            sign_collector |= 1 << (unsafe { std::mem::transmute::<f32, u32>(line.d.x * point.y - line.d.y * point.x - line.d_cross_p0) } >> 31);
        }

        sign_collector != 0 && sign_collector != 3//((sign_collector & 0x1) ^ (sign_collector >> 1)) != 0
    } // pub fn is_in
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\
            points: {:?}\n\
            edges: {:?}\n\
            bounds: [{}; {}]\n\
            ",
            self.points, self.edge_types, self.floor, self.ceiling,
        ))
    }
}


/// Map representation structure
pub struct Map {
    sectors: Vec<Sector>,
    pub camera_location: Vec2f,
    pub camera_height: f32,
    pub camera_rotation: f32,
} // struct Map

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct SectorId(u32);

impl SectorId {
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl Map {
    pub fn find_sector(&self, location: Vec2f) -> Option<SectorId> {
        self.sectors
            .iter()
            .enumerate()
            .find(|(_, sector)| sector.contains(location))
            .map(|(index, _)| SectorId::new(index as u32))
    }

    pub fn find_adjacent_sector(&self, location: Vec2f, adjacent_for: SectorId) -> Option<SectorId> {
        let sector = match self.get_sector(adjacent_for) {
            Some(sector) => sector,
            None => return None,
        };

        if sector.contains(location) {
            return Some(adjacent_for);
        }

        // Try find in adjoint edges or just find if not
        sector.edge_types
            .iter()
            .filter_map(|sector| if let EdgeType::Portal(id) = sector {
                Some(id)
            } else {
                None
            })
            .filter_map(|id| self.sectors
                .get(id.as_u32() as usize)
                .map(|sector| (id, sector)))
            .find(|(_, sector)| sector.contains(location))
            .map(|(id, _)| *id)
    }

    pub fn find_sector_from_old(&self, location: Vec2f, old_sector: SectorId) -> Option<SectorId> {
        self.find_adjacent_sector(location, old_sector)
            .or_else(|| self.find_sector(location))
    }

    pub fn get_sector(&self, id: SectorId) -> Option<&Sector> {
        self.sectors.get(id.as_u32() as usize)
    }
}

/// Map loading error representation structure
#[derive(Debug)]
pub enum WmtLoadingError {
    NumberParsingError,
    UnknownLineType(String),
    InvalidPointIndex(u32),
    NotEnoughPointCoordinates,
    NotEnoughCameraParameters,
    NotEnoughSectorVertices,
    NoAdjointSectorForPortal {
        from_sector: u32,
        by_indices: (u32, u32),
    },
    InvalidSectorBounds {
        floor: f32,
        ceiling: f32
    },
    Other(String),
} // enum WmtLoadingError

impl Map {
    /// Map from .wmt file loading function
    /// * `source` - file text
    /// * Returns valid Map or WmtLoadingError
    pub fn load_from_wmt(source: &str) -> Result<Map, WmtLoadingError> {
        struct SectorData {
            pub point_indices: Vec<u32>,
            pub floor: f32,
            pub ceiling: f32,
            pub walls: HashSet<UnorderedPair<u32>>,
        }
        let mut points = Vec::<Vec2f>::new();
        let mut walls = HashSet::<UnorderedPair<u32>>::new();
        let mut sectors = Vec::<SectorData>::new();
        let mut camera_location = Vec2f {
            x: 0.0,
            y: 0.0,
        };
        let mut camera_height = 0.0;
        let mut camera_rotation = 0.0;

        // Parse file data
        for line in source.lines().map(|line| line.trim()) {
            let mut elem = line.split(' ');

            let line_type = match elem.next() {
                Some(s) => s,
                None => continue,
            };

            match line_type {
                // Comment | Empty line
                "#" | "" => {}

                // Point
                "p" | "point" => {
                    let (sx, sy) = elem.next().zip(elem.next()).ok_or(WmtLoadingError::NotEnoughPointCoordinates)?;
                    let (x, y) = sx.parse::<f32>().ok().zip(sy.parse::<f32>().ok()).ok_or(WmtLoadingError::NumberParsingError)?;

                    points.push(Vec2f { x, y });
                }

                // Wall
                "w" | "wall" => {
                    let (sx, sy) = elem.next()
                        .zip(elem.next())
                        .ok_or(WmtLoadingError::NotEnoughPointCoordinates)?;

                    walls.insert(sx.parse::<u32>().ok()
                        .zip(sy.parse::<u32>().ok())
                        .ok_or(WmtLoadingError::NumberParsingError)?
                        .into()
                    );
                }

                // Sector
                "s" | "sector" => {
                    let (sx, sy) = elem.next().zip(elem.next()).ok_or(WmtLoadingError::NotEnoughPointCoordinates)?;
                    let (floor, ceiling) = sx.parse::<f32>().ok().zip(sy.parse::<f32>().ok()).ok_or(WmtLoadingError::NumberParsingError)?;

                    let point_indices: Vec<u32> = elem
                        .map(|index_str| index_str.parse::<u32>().ok())
                        .collect::<Option<Vec<u32>>>().ok_or(WmtLoadingError::NumberParsingError)?;

                    if point_indices.len() < 3 {
                        return Err(WmtLoadingError::NotEnoughSectorVertices);
                    }

                    let walls = point_indices.iter()
                        .zip({
                            let mut iter = point_indices.iter().cycle();
                            iter.next();
                            iter
                        })
                        .map(|(first, second)| {
                            UnorderedPair::new(*first, *second)
                        })
                        .collect::<HashSet<UnorderedPair<u32>>>();

                    sectors.push(SectorData { point_indices, floor, ceiling, walls });
                }

                "c" | "camera" => {
                    let (((scx, scy), scz), sca) = elem.next().zip(elem.next()).zip(elem.next()).zip(elem.next()).ok_or(WmtLoadingError::NotEnoughCameraParameters)?;
                    (((camera_location.x, camera_location.y), camera_height), camera_rotation) = scx.parse::<f32>().ok()
                        .zip(scy.parse::<f32>().ok())
                        .zip(scz.parse::<f32>().ok())
                        .zip(sca.parse::<f32>().ok())
                        .ok_or(WmtLoadingError::NumberParsingError)?;
                }

                _ => return Err(WmtLoadingError::UnknownLineType(line_type.to_string()))
            }
        }

        // Parse data parsed from file
        let sectors = sectors.iter()
            .enumerate()
            .map(|(sector_index, sector)| -> Result<Sector, WmtLoadingError> {
                let points = sector.point_indices.iter()
                    .map(|index| points
                        .get(*index as usize)
                        .map(|v| *v)
                        .ok_or(WmtLoadingError::InvalidPointIndex(*index))
                    )
                    .collect::<Result<Vec<Vec2f>, WmtLoadingError>>()?;

                let edges = sector.point_indices.iter()
                    .zip({
                        let mut iter = sector.point_indices.iter().cycle();
                        iter.next();
                        iter
                    })
                    .map(|(first, second)| {
                        let pair = UnorderedPair::new(*first, *second);

                        // Find adjoint sector
                        if walls.contains(&pair) {
                            Ok(EdgeType::Wall)
                        } else {
                            let adjoint = sectors.iter()
                                .enumerate()
                                .find(|(index, sector)| *index != sector_index && sector.walls.contains(&pair))
                                .map(|(index, _)| index as u32)
                                .ok_or(WmtLoadingError::NoAdjointSectorForPortal {
                                    from_sector: sector_index as u32,
                                    by_indices: pair.into()
                                })?;

                            Ok(EdgeType::Portal(SectorId::new(adjoint)))
                        }
                    })
                    .collect::<Result<Vec<EdgeType>, WmtLoadingError>>()?;

                // Validate sector bounds
                if sector.floor > sector.ceiling {
                    return Err(WmtLoadingError::InvalidSectorBounds { floor: sector.floor, ceiling: sector.ceiling })
                }

                let mut sector = Sector {
                    points,
                    edge_types: edges,
                    edges: Vec::new(),
                    floor: sector.floor,
                    ceiling: sector.ceiling,
                };
                sector.edges = sector.build_edges();

                Ok(sector)
            })
            .collect::<Result<Vec<Sector>, WmtLoadingError>>()?;

        Ok(Map { sectors, camera_location, camera_height, camera_rotation })
    } // fn load_from_wmt

    /// Iterator through indexed sectors getting function
    /// * Returns DoublEndedIterator with SectorId and &Sector items.E
    pub fn iter_indexed_sectors<'a>(&'a self) -> impl DoubleEndedIterator<Item = (SectorId, &'a Sector)> {
        self.sectors.iter().enumerate().map(|(index, sector)| (SectorId::new(index as u32), sector))
    }
} // impl Map
