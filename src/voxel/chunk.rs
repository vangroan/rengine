use crate::voxel::{ChunkCoord, VoxelCoord, VoxelData};
use specs::{Component, DenseVecStorage};
use std::ops;

/// Length of each side of a chunk
pub const CHUNK_DIM8: usize = 8;

/// Total number of voxels in a chunk
pub const CHUNK_SIZE8: usize = CHUNK_DIM8 * CHUNK_DIM8 * CHUNK_DIM8;

/// Given a global voxel coordinate, return
/// the chunk coordinate that contains it.
pub fn voxel_to_chunk(v: &VoxelCoord) -> ChunkCoord {
    // Integer division truncates, meaning negative
    // numbers round towards 0, so we need to do
    // a pass through floating point maths to get
    // floor behaviour.
    ChunkCoord {
        i: (v.i as f32 / CHUNK_DIM8 as f32).floor() as i32,
        j: (v.j as f32 / CHUNK_DIM8 as f32).floor() as i32,
        k: (v.k as f32 / CHUNK_DIM8 as f32).floor() as i32,
    }
}

/// Interface for a chunk, which acts as storage for
/// voxels.
///
/// Chunk is identified by an index, which is its
/// 3-dimensional coordinate in the chunk space.
///
/// Since a chunk is aware of its own index, and the
/// size of all chunks, it is expected to know its
/// position in global voxel coordinates as well.
pub trait VoxelChunk<D: VoxelData> {
    /// Unique identifier and 3D position
    /// in chunk space.
    fn index(&self) -> &ChunkCoord;

    /// Returns the dimension of the chunk.
    ///
    /// The length returned is used by each of the
    /// three axes.
    fn dim(&self) -> usize;

    /// Position of chunk in global voxel space.
    ///
    /// This position is should be located in the
    /// left, bottom, back corner of the chunk,
    /// which is (0, 0, 0) in local coordinates.
    fn voxel_offset(&self) -> &VoxelCoord;

    /// Checks whether the given global voxel
    /// coordinates are contained within the
    /// bounds of the chunk.
    fn in_bounds<V: Into<VoxelCoord>>(&self, coord: V) -> bool;

    /// Checks whether the given local voxel
    /// coordinates are contained within the
    /// bounds of the chunk.
    fn in_bounds_local<V: Into<VoxelCoord>>(&self, coord: V) -> bool;

    /// Retrieve voxel data at the given coordinate.
    ///
    /// Returns `None` when coordinate is outside of
    /// the chunks bounds.
    fn get<V: Into<VoxelCoord>>(&self, coord: V) -> Option<&D>;

    /// Retrieve voxel data at the given local coordinate.
    fn get_local<V: Into<VoxelCoord>>(&self, coord: V) -> Option<&D>;

    /// Retrieve mutable voxel data at the given coordinate.
    ///
    /// Returns `None` when coordinate is outside of
    /// the chunk's bounds.
    fn get_mut<V: Into<VoxelCoord>>(&mut self, coord: V) -> Option<&mut D>;

    /// Sets the voxel data at the given coordinate.
    fn set<V: Into<VoxelCoord>>(&mut self, coord: V, data: D);
}

/// Trait describing a chunk that keeps adjacency
/// information for each voxel.
///
/// Each voxel record has an adjacency mapping that
/// indicates whether its neighbours are occupied or
/// empty. Occupancy from neighbouring *chunks* is not
/// automatically controlled, and must be enforced by
/// an upper container that has knowledge of chunk
/// layout.
///
/// Adjacency information can be used to occlude cuboid
/// faces that are obstructed by neighouring voxels, and
/// thus don't have to be drawn.
///
/// By bookkeeping the adjacency information, the time
/// taken is sinked during updating of the chunk, freeing
/// up the iteration from performing neighbour lookups.
pub trait MaskedChunk {
    /// Retrieve the adjacency mask for a voxel coordinate.
    ///
    /// Returns `None` when coordinate is outside of
    /// the chunk's bounds.
    fn mask<V: Into<VoxelCoord>>(&self, coord: V) -> Option<VoxelAdjacencyMask>;

    /// Retrieve the adjacency mask for the local voxel coordinate.
    fn mask_local<V: Into<VoxelCoord>>(&self, coord: V) -> Option<VoxelAdjacencyMask>;
}

/// Stores the occupancy information for
/// the 26 surrounding neighbours in
/// 3-dimensions.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoxelAdjacencyMask(u32);

const MASK_BACK: VoxelAdjacencyMask = VoxelAdjacencyMask(0b_0001_0000);
const MASK_FRONT: VoxelAdjacencyMask = VoxelAdjacencyMask(0b_0100_0000_0000_0000_0000_0000);
const MASK_LEFT: VoxelAdjacencyMask = VoxelAdjacencyMask(0b_0001_0000_0000_0000);
const MASK_RIGHT: VoxelAdjacencyMask = VoxelAdjacencyMask(0b_0100_0000_0000_0000);
const MASK_BOTTOM: VoxelAdjacencyMask = VoxelAdjacencyMask(0b_1000_0000_0000);
const MASK_TOP: VoxelAdjacencyMask = VoxelAdjacencyMask(0b_0100_0000_0000_0000_0000_0000);

impl VoxelAdjacencyMask {
    #[inline]
    pub fn is_back(&self) -> bool {
        *self & MASK_BACK == MASK_BACK
    }

    #[inline]
    pub fn is_front(&self) -> bool {
        *self & MASK_FRONT == MASK_FRONT
    }

    #[inline]
    pub fn is_left(&self) -> bool {
        *self & MASK_LEFT == MASK_LEFT
    }

    #[inline]
    pub fn is_right(&self) -> bool {
        *self & MASK_RIGHT == MASK_RIGHT
    }

    #[inline]
    pub fn is_bottom(&self) -> bool {
        *self & MASK_BOTTOM == MASK_BOTTOM
    }

    #[inline]
    pub fn is_top(&self) -> bool {
        *self & MASK_TOP == MASK_TOP
    }
}

impl ops::BitOr for VoxelAdjacencyMask {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        VoxelAdjacencyMask(self.0 | rhs.0)
    }
}

impl ops::BitAnd for VoxelAdjacencyMask {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        VoxelAdjacencyMask(self.0 & rhs.0)
    }
}

impl ops::BitOrAssign for VoxelAdjacencyMask {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl ops::BitAndAssign for VoxelAdjacencyMask {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl ops::Not for VoxelAdjacencyMask {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        VoxelAdjacencyMask(!self.0)
    }
}

/// Given a voxel coordinate offset, create the bit mask
/// where the appropriate bit is set to occupied.
///
/// Supports offsets for immediate neighbours (Moore neighbourhood
/// radius = 1)
///
/// ## Implementation
///
/// This function needs some explaining.
///
/// Imagine a voxel is contained inside an imaginary 3 x 3 x 3 cuboid, containing
/// 27 voxels. Our voxel is in the center, at coordinate (0, 0, 0) surrounded by
/// 26 other voxels which form it's immediate neighbourhood.
///
/// A 2D cross section would look like this:
///
/// ```ignore
/// +-----+-----+-----+
/// | -1  |  0  |  1  |
/// |  1  |  1  |  1  |
/// |  0  |  0  |  0  |
/// +-----+-----+-----+
/// | -1  |  0  |  1  |
/// |  0  |  0  |  0  |
/// |  0  |  0  |  0  |
/// +-----+-----+-----+
/// | -1  |  0  |  1  |
/// | -1  | -1  | -1  |
/// |  0  |  0  |  0  |
/// +-----+-----+-----+
/// ```
///
/// To get a single number index between 0 and 27 we transpose the coordinates of the
/// neighbourhood so the origin would be in a corner, and all coordinates would be
/// positive.
///
/// ```ignore
/// +-----+-----+-----+
/// |  0  |  1  |  2  |
/// |  2  |  2  |  2  |
/// |  0  |  0  |  0  |
/// +-----+-----+-----+
/// |  0  |  1  |  2  |
/// |  1  |  1  |  1  |
/// |  0  |  0  |  0  |
/// +-----+-----+-----+
/// |  0  |  1  |  2  |
/// |  0  |  0  |  0  |
/// |  0  |  0  |  0  |
/// +-----+-----+-----+
/// ```
///
/// From here we can use simple arithmetic to find the index, much like how voxel
/// coordinates would be stored in an array.
///
/// ```ignore
/// let index = x + (y * width) + (z * width * height);
/// ```
///
/// Since the neighbourhood has a small, finite number of neighbours, the index is
/// used as a bit position. The bit is stored in an integer type large enough
/// to hold 27 bits.
fn create_mask(voxel_offset: &[i32; 3]) -> VoxelAdjacencyMask {
    // Translate center to bottom, left, back. Coordinate (-1, -1, -1)
    // will become (0, 0, 0).
    let trans = [
        voxel_offset[0] + 1,
        voxel_offset[1] + 1,
        voxel_offset[2] + 1,
    ];

    // Neighbourhood is treated as a 3x3x3 cube
    let index = trans[0] + trans[1] * 3 + trans[2] * 3 * 3;

    VoxelAdjacencyMask(1 << index)
}

/// Implementation of `VoxelChunk` that naively keeps
/// data in an array. Data is tightly packed, but takes
/// up more memory.
///
/// No deduplication or compression is applied to the
/// data.
#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct VoxelArrayChunk<D: 'static + VoxelData + Sync + Send> {
    /// Unique identifier for this chunk.
    coord: ChunkCoord,

    /// Global position of the bottom, left,
    /// back voxel. Coordinate (0, 0, 0) in
    /// the chunk's local space.
    voxel_offset: VoxelCoord,

    /// Voxel data packed with adjacency map,
    /// describing whether neighbours are occupied
    /// or empty.
    data: [(VoxelAdjacencyMask, D); CHUNK_SIZE8],
}

impl<D> VoxelArrayChunk<D>
where
    D: 'static + VoxelData + Sync + Send,
{
    pub fn new<V>(coord: V) -> Self
    where
        V: Into<ChunkCoord>,
        D: Default + Copy,
    {
        let chunk_coord = coord.into();

        // Translate chunk coordinates to voxel coordinates
        let voxel_offset = VoxelCoord::new(
            chunk_coord.i * CHUNK_DIM8 as i32,
            chunk_coord.j * CHUNK_DIM8 as i32,
            chunk_coord.k * CHUNK_DIM8 as i32,
        );

        VoxelArrayChunk {
            coord: chunk_coord,
            voxel_offset,
            data: [Default::default(); CHUNK_SIZE8],
        }
    }

    fn data_index(&self, local_coord: &VoxelCoord) -> usize {
        (local_coord.i
            + local_coord.j * CHUNK_DIM8 as i32
            + local_coord.k * CHUNK_DIM8 as i32 * CHUNK_DIM8 as i32) as usize
    }
}

impl<D> VoxelChunk<D> for VoxelArrayChunk<D>
where
    D: 'static + VoxelData + Sync + Send,
{
    #[inline]
    fn index(&self) -> &ChunkCoord {
        &self.coord
    }

    #[inline]
    fn dim(&self) -> usize {
        CHUNK_DIM8
    }

    fn voxel_offset(&self) -> &VoxelCoord {
        &self.voxel_offset
    }

    fn in_bounds<V>(&self, coord: V) -> bool
    where
        V: Into<VoxelCoord>,
    {
        let VoxelCoord { i, j, k } = coord.into();
        let (i1, j1, k1) = self.voxel_offset.clone().into();
        let (i2, j2, k2) = (
            i1 + CHUNK_DIM8 as i32,
            j1 + CHUNK_DIM8 as i32,
            k1 + CHUNK_DIM8 as i32,
        );

        i >= i1 && j >= j1 && k >= k1 && i < i2 && j < j2 && k < k2
    }

    fn in_bounds_local<V>(&self, coord: V) -> bool
    where
        V: Into<VoxelCoord>,
    {
        let VoxelCoord { i, j, k } = coord.into();
        let dim = self.dim() as i32;
        i >= 0 && j >= 0 && k >= 0 && i < dim && j < dim && k < dim
    }

    fn get<V>(&self, coord: V) -> Option<&D>
    where
        V: Into<VoxelCoord>,
    {
        let voxel_coord: VoxelCoord = coord.into();

        if self.in_bounds(voxel_coord.clone()) {
            // Convert to local space
            let local_coord = voxel_coord - &self.voxel_offset;
            let index = self.data_index(&local_coord);

            self.data.get(index).map(|el| &el.1)
        } else {
            None
        }
    }

    fn get_local<V>(&self, coord: V) -> Option<&D>
    where
        V: Into<VoxelCoord>,
    {
        let local_coord: VoxelCoord = coord.into();

        if self.in_bounds_local(local_coord.clone()) {
            let index = self.data_index(&local_coord);

            self.data.get(index).map(|el| &el.1)
        } else {
            None
        }
    }

    fn get_mut<V>(&mut self, coord: V) -> Option<&mut D>
    where
        V: Into<VoxelCoord>,
    {
        let voxel_coord: VoxelCoord = coord.into();

        if self.in_bounds(voxel_coord.clone()) {
            // Convert to local space
            let local_coord = voxel_coord - &self.voxel_offset;
            let index = self.data_index(&local_coord);

            self.data.get_mut(index).map(|el| &mut el.1)
        } else {
            None
        }
    }

    fn set<V>(&mut self, coord: V, data: D)
    where
        V: Into<VoxelCoord>,
    {
        let voxel_coord: VoxelCoord = coord.into();

        // Convert to local space
        let local_coord = voxel_coord.clone() - &self.voxel_offset;
        let center_index = self.data_index(&local_coord);
        let occupied = data.occupied();

        if self.in_bounds(voxel_coord) {
            self.data[center_index] = (Default::default(), data);
        }

        // Regardless whether the coordinate is in bounds or
        // not, we set the adjacency masks of surrounding voxels.
        //
        // Allows for adjacency information to keep up-to-date when
        // neighbouring chunks are updated.
        //
        // Iterate neighbourhood, with given coordinate as the center.
        for x in -1..2 {
            for y in -1..2 {
                for z in -1..2 {
                    // We don't consider the given coordinate. It will
                    // be updated if its neighbours are updated later.
                    if [x, y, z] == [0, 0, 0] {
                        continue;
                    }

                    // Set the neighbour's mask according to whether the center
                    // is occupied.
                    let neigh_coord = local_coord.clone() + [x, y, z].into();
                    let index = self.data_index(&neigh_coord);
                    if let Some(voxel_bundle) = self.data.get_mut(index) {
                        // Prepare a mask from the perspective of the neighbour.
                        let center_as_neighbour = [-x, -y, -z];
                        let mask = create_mask(&center_as_neighbour);

                        if occupied {
                            voxel_bundle.0 |= mask;
                        } else {
                            voxel_bundle.0 &= !mask;
                        }
                    }
                }
            }
        }
    }
}

impl<D> MaskedChunk for VoxelArrayChunk<D>
where
    D: 'static + VoxelData + Sync + Send,
{
    fn mask<V>(&self, coord: V) -> Option<VoxelAdjacencyMask>
    where
        V: Into<VoxelCoord>,
    {
        let voxel_coord: VoxelCoord = coord.into();

        if self.in_bounds(voxel_coord.clone()) {
            let local_coord = voxel_coord - &self.voxel_offset;
            let index = self.data_index(&local_coord);

            self.data.get(index).map(|el| el.0)
        } else {
            None
        }
    }

    fn mask_local<V>(&self, coord: V) -> Option<VoxelAdjacencyMask>
    where
        V: Into<VoxelCoord>,
    {
        let local_coord: VoxelCoord = coord.into();

        if self.in_bounds_local(local_coord.clone()) {
            let index = self.data_index(&local_coord);

            self.data.get(index).map(|el| el.0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_voxel_to_chunk() {
        let v1 = VoxelCoord::new(7, 6, 5);
        assert_eq!(ChunkCoord::new(0, 0, 0), voxel_to_chunk(&v1));

        let v2 = VoxelCoord::new(7, 8, 5);
        assert_eq!(ChunkCoord::new(0, 1, 0), voxel_to_chunk(&v2));

        let v3 = VoxelCoord::new(-7, 8, 5);
        assert_eq!(ChunkCoord::new(-1, 1, 0), voxel_to_chunk(&v3));
    }

    #[test]
    fn test_create_mask() {
        let m_bottom_left_back = create_mask(&[-1, -1, -1]);
        assert_eq!(
            VoxelAdjacencyMask(0b_0000_0000_0000_0000_0000_0000_0000_0001),
            m_bottom_left_back
        );

        let m_middle_left_back = create_mask(&[0, -1, -1]);
        assert_eq!(
            VoxelAdjacencyMask(0b_0000_0000_0000_0000_0000_0000_0000_0010),
            m_middle_left_back
        );

        let m_center = create_mask(&[0, 0, 0]);
        assert_eq!(
            VoxelAdjacencyMask(0b_0000_0000_0000_0000_0010_0000_0000_0000),
            m_center
        );

        let m_top_right_front = create_mask(&[1, 1, 1]);
        assert_eq!(
            VoxelAdjacencyMask(0b_0000_0100_0000_0000_0000_0000_0000_0000),
            m_top_right_front
        );
    }

    #[test]
    fn test_mask_eq() {
        // println!("Front: {:b}", create_mask(&[0, 0, 1]).0);
        // println!("Back: {:b}", create_mask(&[0, 0, -1]).0);
        // println!("Left: {:b}", create_mask(&[-1, 0, 0]).0);
        // println!("Right: {:b}", create_mask(&[1, 0, 0]).0);
        // println!("Bottom: {:b}", create_mask(&[1, -1, 0]).0);
        // println!("Top: {:b}", create_mask(&[0, 1, 0]).0);
        let m_front = create_mask(&[0, 0, 1]);
        assert!(m_front.is_front());

        let m_back = create_mask(&[0, 0, -1]);
        assert!(m_back.is_back());
    }
}
