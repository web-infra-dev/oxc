use std::mem::{align_of, size_of};

#[derive(Clone, Default, Debug)]
pub struct Layout {
    pub layout_64: PlatformLayout,
    pub layout_32: PlatformLayout,
}

impl Layout {
    pub fn from_type<T>() -> Self {
        Self::from_size_align(
            u32::try_from(size_of::<T>()).unwrap(),
            u32::try_from(align_of::<T>()).unwrap(),
        )
    }

    pub fn from_size_align(size: u32, align: u32) -> Self {
        Self {
            layout_64: PlatformLayout::from_size_align(size, align),
            layout_32: PlatformLayout::from_size_align(size, align),
        }
    }

    pub fn from_size_align_niche(size: u32, align: u32, niche: Niche) -> Self {
        Self {
            layout_64: PlatformLayout::from_size_align_niche(size, align, niche.clone()),
            layout_32: PlatformLayout::from_size_align_niche(size, align, niche),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.layout_64.align != 0
    }
}

#[derive(Clone, Default, Debug)]
pub struct PlatformLayout {
    pub size: u32,
    pub align: u32,
    pub niche: Option<Niche>,
}

impl PlatformLayout {
    pub fn from_size_align(size: u32, align: u32) -> Self {
        Self { size, align, niche: None }
    }

    pub fn from_size_align_niche(size: u32, align: u32, niche: Niche) -> Self {
        Self { size, align, niche: Some(niche) }
    }
}

#[derive(Clone, Debug)]
pub struct Niche {
    // Byte offset of the niche from start of type
    pub offset: u32,
    // Size of the niche in bytes
    #[expect(dead_code)]
    pub size: u32,
    // `true` if niche is at start of range
    #[expect(dead_code)]
    pub is_range_start: bool,
    // Number of niches
    pub count: u32,
}

impl Niche {
    pub fn new(offset: u32, size: u32, is_range_start: bool, count: u32) -> Self {
        Self { offset, size, is_range_start, count }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Offset {
    pub offset_64: u32,
    pub offset_32: u32,
}
