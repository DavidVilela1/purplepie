//! Parts of a texture: sprite-sheet cells and tiles (ADR-020 extension, PP-019).
//!
//! Plain data in texel coordinates. The draw list turns a region into the
//! instance's `uv_rect`; the GPU path is the one sprites and glyphs already use.

use crate::math::Vec2;

/// A rectangle of texels inside a texture: what a [`Sprite`](super::Sprite)
/// shows when it has a region (one frame of a sprite sheet, one tile).
///
/// Texel coordinates like an image editor's: `(x, y)` is the top-left texel,
/// rows go **down**. A region that sticks out of its texture is cut to the
/// texture's edges; one that lies completely outside draws nothing.
///
/// ```
/// use purplepie::math::Vec2;
/// use purplepie::render::TextureRegion;
///
/// // The 16×16 texels starting 32 texels from the left of the top row.
/// let region = TextureRegion::new(32, 0, 16, 16);
/// assert_eq!(region.size(), Vec2::new(16.0, 16.0));
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextureRegion {
    /// Left edge, in texels from the texture's left side.
    pub x: u32,
    /// Top edge, in texels from the texture's top row.
    pub y: u32,
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
}

impl TextureRegion {
    /// The `width`×`height` texels whose top-left texel is (`x`, `y`).
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Width and height in texels, e.g. to give a sprite its natural size.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// The region cut to a `texture_width`×`texture_height` texture, as
    /// `[u, v, width, height]` in texture coordinates (0..1, v down), or
    /// `None` if nothing of it lies inside the texture.
    pub(crate) fn uv_rect(&self, texture_width: u32, texture_height: u32) -> Option<[f32; 4]> {
        let right = self.x.saturating_add(self.width).min(texture_width);
        let bottom = self.y.saturating_add(self.height).min(texture_height);
        if self.x >= right || self.y >= bottom {
            return None;
        }
        let (w, h) = (texture_width as f32, texture_height as f32);
        Some([
            self.x as f32 / w,
            self.y as f32 / h,
            (right - self.x) as f32 / w,
            (bottom - self.y) as f32 / h,
        ])
    }
}

/// A sprite sheet laid out as a grid of equal cells: finds the
/// [`TextureRegion`] of a cell by column and row, or by frame number.
///
/// `margin` is the empty border around the whole grid and `spacing` the gap
/// between neighbouring cells, both in texels. Frames are numbered row by
/// row, left to right, starting at 0.
///
/// ```
/// use purplepie::render::{SpriteGrid, TextureRegion};
///
/// // 4 columns × 2 rows of 16×16 frames, 1 texel apart, 2 texels from the edge.
/// let grid = SpriteGrid::new(16, 16, 4, 2).with_spacing(1).with_margin(2);
/// assert_eq!(grid.len(), 8);
/// assert_eq!(grid.frame(5), Some(TextureRegion::new(2 + 17, 2 + 17, 16, 16)));
/// assert_eq!(grid.cell(1, 1), grid.frame(5));
/// assert_eq!(grid.frame(8), None);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SpriteGrid {
    /// Width of one cell in texels.
    pub cell_width: u32,
    /// Height of one cell in texels.
    pub cell_height: u32,
    /// Number of cells per row.
    pub columns: u32,
    /// Number of rows.
    pub rows: u32,
    /// Gap between neighbouring cells, in texels.
    pub spacing: u32,
    /// Empty border around the grid, in texels.
    pub margin: u32,
}

impl SpriteGrid {
    /// `columns` × `rows` cells of `cell_width`×`cell_height` texels each,
    /// packed edge to edge from the top-left texel (no spacing, no margin).
    pub const fn new(cell_width: u32, cell_height: u32, columns: u32, rows: u32) -> Self {
        Self {
            cell_width,
            cell_height,
            columns,
            rows,
            spacing: 0,
            margin: 0,
        }
    }

    /// The same grid with `spacing` texels between cells.
    pub const fn with_spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// The same grid with a `margin`-texel border around it.
    pub const fn with_margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }

    /// Number of cells (`columns × rows`).
    pub const fn len(&self) -> u32 {
        self.columns.saturating_mul(self.rows)
    }

    /// `true` if the grid has no cells.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The cell in `column` and `row` (both from 0), or `None` outside the grid.
    pub fn cell(&self, column: u32, row: u32) -> Option<TextureRegion> {
        if column >= self.columns || row >= self.rows {
            return None;
        }
        let step_x = self.cell_width.checked_add(self.spacing)?;
        let step_y = self.cell_height.checked_add(self.spacing)?;
        Some(TextureRegion::new(
            self.margin.checked_add(column.checked_mul(step_x)?)?,
            self.margin.checked_add(row.checked_mul(step_y)?)?,
            self.cell_width,
            self.cell_height,
        ))
    }

    /// Frame `index`, counting row by row from the top-left cell, or `None`
    /// past the last cell.
    pub fn frame(&self, index: u32) -> Option<TextureRegion> {
        if self.columns == 0 {
            return None;
        }
        self.cell(index % self.columns, index / self.columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_and_partial_regions_map_to_texture_coordinates() {
        assert_eq!(
            TextureRegion::new(0, 0, 32, 16).uv_rect(32, 16),
            Some([0.0, 0.0, 1.0, 1.0])
        );
        assert_eq!(
            TextureRegion::new(8, 8, 8, 8).uv_rect(32, 16),
            Some([0.25, 0.5, 0.25, 0.5])
        );
    }

    #[test]
    fn regions_are_cut_to_the_texture_and_empty_ones_draw_nothing() {
        // Sticks out on the right and bottom: cut to 8×8.
        assert_eq!(
            TextureRegion::new(24, 8, 100, 100).uv_rect(32, 16),
            Some([0.75, 0.5, 0.25, 0.5])
        );
        assert_eq!(TextureRegion::new(32, 0, 8, 8).uv_rect(32, 16), None);
        assert_eq!(TextureRegion::new(0, 16, 8, 8).uv_rect(32, 16), None);
        assert_eq!(TextureRegion::new(0, 0, 0, 8).uv_rect(32, 16), None);
        assert_eq!(
            TextureRegion::new(u32::MAX, 0, u32::MAX, 1).uv_rect(32, 16),
            None
        );
    }

    #[test]
    fn grid_cells_follow_margin_and_spacing() {
        let grid = SpriteGrid::new(8, 4, 3, 2).with_spacing(2).with_margin(1);
        assert_eq!(grid.len(), 6);
        assert_eq!(grid.cell(0, 0), Some(TextureRegion::new(1, 1, 8, 4)));
        assert_eq!(
            grid.cell(2, 0),
            Some(TextureRegion::new(1 + 2 * 10, 1, 8, 4))
        );
        assert_eq!(grid.cell(1, 1), Some(TextureRegion::new(11, 1 + 6, 8, 4)));
        assert_eq!(grid.cell(3, 0), None);
        assert_eq!(grid.cell(0, 2), None);
        let frames: Vec<_> = (0..7).map(|i| grid.frame(i)).collect();
        assert_eq!(frames[3], grid.cell(0, 1), "frames go row by row");
        assert_eq!(frames[6], None);
    }

    #[test]
    fn degenerate_grids_have_no_cells() {
        let empty = SpriteGrid::new(8, 8, 0, 4);
        assert!(empty.is_empty());
        assert_eq!(empty.frame(0), None);
        assert_eq!(SpriteGrid::default().cell(0, 0), None);
        // Overflowing positions are reported as missing, not wrapped around.
        let huge = SpriteGrid::new(u32::MAX, 1, 3, 1);
        assert!(huge.cell(1, 0).is_some());
        assert_eq!(huge.cell(2, 0), None);
    }
}
