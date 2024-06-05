/// WEIRD Project
/// `File` map.rs
/// `Description` Map implementation module
/// `Author` TioT2
/// `Last changed` 04.05.2024

use std::collections::BTreeMap;
use crate::math::*;

/// Sector type representation structure
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EdgeType {
    /// Wall
    Wall,
    /// Portal to some sector
    Portal {
        dst_sector_id: SectorId,
    },
} // enum Edge

/// Edge math data container
#[derive(Copy, Clone, Debug)]
pub struct Edge {
    /// Edge first point position
    pub p0: Vec2f,
    /// Edge second point position
    pub p1: Vec2f,
    /// p0 -> p1 direction
    pub direction: Vec2f,
    /// d and p0 cross product
    pub d_cross_p0: f32,
    /// Edge type
    pub ty: EdgeType,
} // struct Edge

impl Edge {
    /// New edge create function
    /// * `p0` - first point
    /// * `p1` - second point
    /// * `ty` - edge type
    /// * Returns new edge
    pub fn new(p0: Vec2f, p1: Vec2f, ty: EdgeType) -> Self {
        let direction = p1 - p0;
        Self { p0, p1, d_cross_p0: direction % p0, direction, ty }
    } // fn new

    /// Build edge loop from points
    /// * `points` - point iterator
    /// * Returns edge iterator
    pub fn loop_from_points(points: impl Iterator<Item = (Vec2f, EdgeType)> + Clone) -> impl Iterator<Item = Edge> {
        points
            .clone()
            .zip(points.map(|t| t.0).cycle().skip(1))
            .map(|((left, ty), right)| Edge::new(left, right, ty))
    } // fn loop_from_points
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wall => f.write_str("Wall"),
            Self::Portal{dst_sector_id} => f.write_fmt(format_args!("Portal({})", dst_sector_id.as_u32()))
        }
    } // fn fmt
} // impl std::fmt::Display for Edge

/// Sector representation structure
pub struct Sector {
    /// Sector edge set
    pub edges: Vec<Edge>,
    /// Floor height
    pub floor: f32,
    /// Ceiling height
    pub ceiling: f32,
} // struct Sector

impl Sector {
    /// Sector with loop of walls representation structure
    /// * `points` - set of points forming sector, points polygon convexity required
    /// * Returns sector with `points` points
    pub fn wall_loop(points: impl Iterator<Item = Vec2f> + Clone) -> Self {
        Self {
            edges: Edge::loop_from_points(points.map(|v| (v, EdgeType::Wall))).collect(),
            floor: 0.0,
            ceiling: 1.0,
        }
    } // fn wall_loop

    /// Check for point being located in sector
    /// * `point` - point to test
    /// * Returns true if this point is contained in the sector
    pub fn contains(&self, point: Vec2f) -> bool {
        let sign_collector = self.edges.iter().fold(0u8, |state, line| state | (1 << (unsafe { std::mem::transmute::<f32, u32>(line.direction.x * point.y - line.direction.y * point.x - line.d_cross_p0) } >> 31)));

        sign_collector != 0 && sign_collector != 3//((sign_collector & 0x1) ^ (sign_collector >> 1)) != 0
    } // pub fn is_in
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\
            edges: {:?}\n\
            bounds: [{}; {}]\n\
            ",
            self.edges, self.floor, self.ceiling,
        ))
    } // fn fmt
} // fn Sector


/// Map camera state representaiton structure
pub struct MapCameraState {
    /// Camera location
    pub location: Vec2f,
    /// Camera height
    pub height: f32,
    /// Camera rotation angle
    pub angle: f32,
} // struct MapCameraState

/// Map representation structure
pub struct Map {
    sectors: Vec<Sector>,

    /// Map camera parameters
    pub camera_location: Vec2f,
    pub camera_height: f32,
    pub camera_rotation: f32,
} // struct Map

/// Map camera info representation structure
pub struct CameraInfo {
    pub location: Vec2f,
    pub height: f32,
    pub rotation: f32,
} // sturct CameraInfo

/// Sector unique identifier represetnation structure
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct SectorId(u32);

impl SectorId {
    /// Sector id from u32 creation function
    /// * `index` - index of sector to turn into SectorID
    /// * Returns new SectorID
    pub fn new(index: u32) -> Self {
        Self(index)
    } // fn new

    /// SectorID into unique underlying U32 turning function
    /// * Returns SectorId unique underlying u32
    pub fn as_u32(self) -> u32 {
        self.0
    } // fn as_u32
} // impl SectorId

impl Map {
    /// Map sector by point finding function
    /// * `location` - point that must be contained in resulting sector
    /// * Returns option of identifier of sector that contains `location` point
    pub fn find_sector(&self, location: Vec2f) -> Option<SectorId> {
        self.sectors
            .iter()
            .enumerate()
            .find(|(_, sector)| sector.contains(location))
            .map(|(index, _)| SectorId::new(index as u32))
    } // fn find_sector

    /// Map sector with adjacent one finding function
    /// * `location` - point that must be contained in resulting sector
    /// * `adjacent_for` - point that resulting sector may be adjacent with
    /// * Returns option of identifier of sector that contains `location` point
    pub fn find_adjacent_sector(&self, location: Vec2f, adjacent_for: SectorId) -> Option<SectorId> {
        let sector = match self.get_sector(adjacent_for) {
            Some(sector) => sector,
            None => return None,
        };

        if sector.contains(location) {
            return Some(adjacent_for);
        }

        // Try find in adjoint edges or just find if not
        sector.edges
            .iter()
            .filter_map(|edge| if let EdgeType::Portal{dst_sector_id} = edge.ty {
                Some(dst_sector_id)
            } else {
                None
            })
            .filter_map(|id| self.sectors
                .get(id.as_u32() as usize)
                .map(|sector| (id, sector)))
            .find(|(_, sector)| sector.contains(location))
            .map(|(id, _)| id)
    } // fn find_adjacent_sector

    /// Trying to find adjacent sector and if not, find any suiting.
    /// * `location` - location of point resulting sector required to contain
    /// * `old_sector` - identifier of sector that may be former sector of the `location` point.
    /// * Returns option of identifier of sector containing `location` point
    pub fn find_sector_from_old(&self, location: Vec2f, old_sector: SectorId) -> Option<SectorId> {
        self.find_adjacent_sector(location, old_sector)
            .or_else(|| self.find_sector(location))
    } // pub fn find_sector_from_old

    /// Sector by identifier getting function
    /// * `id` - sector identifier
    /// * Returns option of required sector reference.
    pub fn get_sector(&self, id: SectorId) -> Option<&Sector> {
        self.sectors.get(id.as_u32() as usize)
    } // fn get_sector
} // impl Map

#[derive(Debug, Clone)]
pub enum Wmt2LoadingError {
    /// Unknown directive
    UnknownDirective(String),

    /// Floating point parsing error
    FloatParsingError(std::num::ParseFloatError),

    /// No enough camera parameter
    NotEnoughCameraParameters,

    /// No sector boundaries at all
    NoSectorBoundaries,

    /// No sector edges starting symbol
    NoSectorEdgesStart,

    /// Not enough points for some kind of coordinates
    NotEnoughPointCoordinates,

    /// Sector with value not
    UnknownSectorReferenced(String),

    /// Some other error
    Other(String),
} //

impl Map {
    pub fn load_from_wmt(source: &str) -> Result<Map, Wmt2LoadingError> {
        enum ChunkType {
            Sector,
            Camera,
            None,
        }
        let mut mode = ChunkType::None;
        let mut camera = CameraInfo {
            location: Vec2f::new(0.0, 0.0),
            height: 0.3,
            rotation: 0.0,
        };

        struct RawSectorPoint {
            base_point: Vec2f,
            dst_sector_name: Option<String>,
        }

        struct RawSector {
            floor: f32,
            ceiling: f32,
            points: Vec<RawSectorPoint>,
        }

        let mut raw_sectors = BTreeMap::new();

        for mut line in source.lines() {
            // Cut comments
            if let Some(i) = line.find("//") {
                line = line.split_at(i).0
            }
            let line = line.chars().filter(|v| !v.is_whitespace()).collect::<String>();

            if line.is_empty() {
                continue;
            }

            if line.starts_with("#") {
                if line.starts_with("#sector") {
                    mode = ChunkType::Sector;
                } else if line.starts_with("#camera") {
                    mode = ChunkType::Camera;
                } else {
                    return Err(Wmt2LoadingError::UnknownDirective(line.get(1..).unwrap().into()))
                }
                continue;
            }

            match mode {
                ChunkType::Camera => {
                    // Parsing camera information in single fucking line
                    [camera.location.x, camera.location.y, camera.height, camera.rotation] = line
                        .chars()
                        .filter(|v| !v.is_whitespace())
                        .collect::<String>()
                        .split(',')
                        .map(|v| v.parse::<f32>())
                        .collect::<Result<Vec<f32>, _>>()
                        .map_err(|e| Wmt2LoadingError::FloatParsingError(e))?
                        .try_into()
                        .map_err(|_| Wmt2LoadingError::NotEnoughCameraParameters)?;
                }
                ChunkType::Sector => {
                    fn parse_pair(pair: &str) -> Result<(f32, f32), Wmt2LoadingError> {
                        let mut s = pair.split('/').map(|v| v.parse::<f32>().map_err(|e| Wmt2LoadingError::FloatParsingError(e)));
                        let (x, y) = s.next().zip(s.next()).ok_or(Wmt2LoadingError::NotEnoughPointCoordinates)?;
                        Ok((x?, y?))
                    }

                    let (sector_name, rest) = line.as_str().split_at(line.find(':').ok_or(Wmt2LoadingError::NoSectorBoundaries)?);
                    let (sector_bounds, rest) = rest[1..].split_at(rest.find('[').ok_or(Wmt2LoadingError::NoSectorEdgesStart)?);
                    let (floor, ceiling) = parse_pair(&sector_bounds.trim_end_matches('['))?;

                    let mut points = Vec::<RawSectorPoint>::new();

                    for pt in rest.trim_end_matches(']').split(',') {
                        let (point_str, dst_sector_name) = pt
                            .find(':')
                            .map(|i| {
                                let (s, t) = pt.split_at(i);
                                (s, Some(t[1..].to_string()))
                            })
                            .unwrap_or((pt, None));

                        points.push(RawSectorPoint {
                            base_point: Vec2f::from_tuple(parse_pair(point_str)?),
                            dst_sector_name,
                        });
                    }

                    raw_sectors.insert(sector_name.to_owned(), RawSector {
                        floor,
                        ceiling,
                        points,
                    });
                }
                _ => {}
            }
        }

        let name_to_index = raw_sectors.keys().enumerate().map(|(a, b)| (b.clone(), SectorId::new(a as u32))).collect::<BTreeMap<String, SectorId>>();

        Ok(Map {
            camera_height: camera.height,
            camera_location: camera.location,
            camera_rotation: camera.rotation,
            sectors: raw_sectors
                .values()
                .map(|sector| Ok(Sector {
                    floor: sector.floor,
                    ceiling: sector.ceiling,
                    edges: Edge::loop_from_points(sector.points.iter().map(|v| (v.base_point, EdgeType::Wall)))
                        .zip(sector.points.iter())
                        .map(|(mut edge, point)| {
                            if let Some(dst_name) = point.dst_sector_name.as_ref() {
                                edge.ty = EdgeType::Portal {
                                    dst_sector_id: name_to_index.get(dst_name.as_str()).copied().ok_or(Wmt2LoadingError::UnknownSectorReferenced(dst_name.into()))?
                                };
                            }
                            Ok(edge)
                        })
                        .collect::<Result<Vec<Edge>, Wmt2LoadingError>>()?,
                }))
            .collect::<Result<Vec<Sector>, Wmt2LoadingError>>()?
        })
    } // fn load_from_wmt

    /// Iterator through indexed sectors getting function
    /// * Returns DoublEndedIterator with SectorId and &Sector items.E
    pub fn iter_indexed_sectors<'a>(&'a self) -> impl DoubleEndedIterator<Item = (SectorId, &'a Sector)> {
        self.sectors.iter().enumerate().map(|(index, sector)| (SectorId::new(index as u32), sector))
    } // fn iter_indexed_sectors
} // impl Map

// file map.rs
