//! The arena: floor, walls, the HUD texts and the camera that follows the player.

use purplepie::Context;
use purplepie::ecs::Entity;
use purplepie::math::{Rect, Rng, Transform2D, Vec2};
use purplepie::render::{
    Color, FontId, Hidden, Layer, Quad, ScreenSpace, Sprite, Text, TextAnchor, TextureOptions,
};

/// The playing field, centred on the origin (world units).
pub const ARENA: Vec2 = Vec2::new(1600.0, 1200.0);
/// Size of one floor tile.
const TILE: f32 = 100.0;
/// Thickness of the walls drawn around the arena.
pub const WALL: f32 = 24.0;

/// Which HUD text an entity is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hud {
    /// Score, wave and time.
    Status,
    /// Hit points.
    Health,
    /// The big centred word (title, pause, game over).
    Banner,
    /// The smaller lines under the banner (what to click or press).
    Prompt,
    /// The dark panel behind the banner and prompt (hidden with them).
    Backdrop,
}

/// The arena as a rectangle.
pub fn arena() -> Rect {
    Rect::from_center_size(Vec2::ZERO, ARENA)
}

/// Spawns the floor, the walls and the HUD texts.
pub fn spawn(ctx: &mut Context<'_>) -> purplepie::Result<FontId> {
    let floor = ctx.load_texture("textures/floor.png")?;
    let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
    let world = ctx.world_mut();

    let (columns, rows) = ((ARENA.x / TILE) as u32, (ARENA.y / TILE) as u32);
    for ty in 0..rows {
        for tx in 0..columns {
            let at = -ARENA / 2.0 + (Vec2::new(tx as f32, ty as f32) + 0.5) * TILE;
            world.spawn((
                Transform2D::from_position(at),
                Sprite::new(floor, Vec2::splat(TILE)),
                Layer(-10),
            ));
        }
    }
    let wall_color = Color::hex(0x7B2CBF);
    let half = ARENA / 2.0 + WALL / 2.0;
    let across = Vec2::new(ARENA.x + 2.0 * WALL, WALL);
    for (at, size) in [
        (Vec2::new(0.0, half.y), across),
        (Vec2::new(0.0, -half.y), across),
        (Vec2::new(half.x, 0.0), Vec2::new(WALL, ARENA.y)),
        (Vec2::new(-half.x, 0.0), Vec2::new(WALL, ARENA.y)),
    ] {
        world.spawn((
            Transform2D::from_position(at),
            Quad::new(size, wall_color),
            Layer(-5),
        ));
    }

    let hud = |text: &str, size: f32| Text::new(text, font, size);
    world.spawn((
        Transform2D::from_position(Vec2::new(16.0, -16.0)),
        hud("PURPLE SWARM", 24.0)
            .with_color(Color::hex(0xC77DFF))
            .with_anchor(TextAnchor::TOP_LEFT),
        ScreenSpace::TOP_LEFT,
        Layer(20),
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(16.0, -48.0)),
        hud("", 18.0).with_anchor(TextAnchor::TOP_LEFT),
        ScreenSpace::TOP_LEFT,
        Layer(20),
        Hud::Status,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(-16.0, -16.0)),
        hud("", 22.0)
            .with_color(Color::hex(0x70E000))
            .with_anchor(TextAnchor::TOP_RIGHT),
        ScreenSpace::TOP_RIGHT,
        Layer(20),
        Hud::Health,
    ));
    world.spawn((
        Transform2D::default(),
        Quad::new(Vec2::new(620.0, 210.0), Color::rgba(0.03, 0.01, 0.08, 0.85)),
        ScreenSpace::CENTER,
        Layer(29),
        Hidden,
        Hud::Backdrop,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, 40.0)),
        hud("", 48.0)
            .with_color(Color::hex(0xC77DFF))
            .with_anchor(TextAnchor::CENTER),
        ScreenSpace::CENTER,
        Layer(30),
        Hud::Banner,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, -36.0)),
        hud("", 20.0).with_anchor(TextAnchor::CENTER),
        ScreenSpace::CENTER,
        Layer(30),
        Hud::Prompt,
    ));
    world.spawn((
        Transform2D::from_position(Vec2::new(0.0, 16.0)),
        hud(
            "WASD move - mouse aim - hold left button to shoot - Esc pauses",
            16.0,
        )
        .with_color(Color::rgba(1.0, 1.0, 1.0, 0.6))
        .with_anchor(TextAnchor::BOTTOM_CENTER),
        ScreenSpace::BOTTOM_CENTER,
        Layer(20),
    ));
    Ok(font)
}

/// The player's sprite, facing +X at rotation 0.
pub fn spawn_player(ctx: &mut Context<'_>, size: f32) -> purplepie::Result<()> {
    let texture = ctx.load_texture_with("textures/player.png", TextureOptions::LINEAR)?;
    ctx.world_mut().spawn((
        Transform2D::default(),
        Sprite::new(texture, Vec2::splat(size)),
        crate::Player,
        Layer(5),
    ));
    Ok(())
}

/// Sets the text of one HUD element.
pub fn set_hud(ctx: &mut Context<'_>, which: Hud, content: &str) {
    for (text, hud) in ctx.world_mut().query_mut::<(&mut Text, &Hud)>() {
        if *hud == which && text.content != content {
            content.clone_into(&mut text.content);
        }
    }
}

/// Sets the colour of one HUD text.
pub fn set_hud_color(ctx: &mut Context<'_>, which: Hud, color: Color) {
    for (text, hud) in ctx.world_mut().query_mut::<(&mut Text, &Hud)>() {
        if *hud == which {
            text.color = color;
        }
    }
}

/// Shows the centred banner with `prompt` under it on a dark panel, or hides
/// all three when `banner` is empty.
pub fn show_banner(ctx: &mut Context<'_>, banner: &str, prompt: &str) {
    set_hud(ctx, Hud::Banner, banner);
    set_hud(ctx, Hud::Prompt, prompt);
    let world = ctx.world_mut();
    let panels: Vec<Entity> = world
        .query::<(Entity, &Hud)>()
        .iter()
        .filter(|(_, hud)| **hud == Hud::Backdrop)
        .map(|(e, _)| e)
        .collect();
    for panel in panels {
        if banner.is_empty() {
            let _ = world.insert_one(panel, Hidden);
        } else {
            let _ = world.remove_one::<Hidden>(panel);
        }
    }
}

/// Keeps the camera on `target` but never shows space outside the walls
/// (when the arena is smaller than the view along an axis, it stays centred).
pub fn follow_camera(ctx: &mut Context<'_>, target: Vec2) {
    let viewport = ctx.viewport_size();
    let camera = ctx.camera_mut();
    let half_view = viewport / (2.0 * camera.effective_zoom());
    let room = (ARENA / 2.0 + WALL - half_view).max(Vec2::ZERO);
    camera.position = target.clamp(-room, room);
}

/// A point on the arena edge, at least `keep_away` from `avoid` (best of a few
/// tries; the farthest one if none qualifies).
pub fn edge_point(rng: &mut Rng, avoid: Vec2, keep_away: f32, inset: f32) -> Vec2 {
    let half = ARENA / 2.0 - Vec2::splat(inset);
    let mut best = Vec2::ZERO;
    let mut best_distance = -1.0;
    for _ in 0..8 {
        let point = match rng.range_u32(0, 4) {
            0 => Vec2::new(rng.range_f32(-half.x, half.x), half.y),
            1 => Vec2::new(rng.range_f32(-half.x, half.x), -half.y),
            2 => Vec2::new(half.x, rng.range_f32(-half.y, half.y)),
            _ => Vec2::new(-half.x, rng.range_f32(-half.y, half.y)),
        };
        let distance = point.distance(avoid);
        if distance >= keep_away {
            return point;
        }
        if distance > best_distance {
            best = point;
            best_distance = distance;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_points_are_on_the_inset_edge_and_away_from_the_player() {
        let mut rng = Rng::new(1);
        let half = ARENA / 2.0 - Vec2::splat(20.0);
        for i in 0..500 {
            let avoid = Vec2::new((i % 7) as f32 * 100.0 - 300.0, 0.0);
            let p = edge_point(&mut rng, avoid, 300.0, 20.0);
            let on_edge = (p.x.abs() - half.x).abs() < 1e-3 || (p.y.abs() - half.y).abs() < 1e-3;
            assert!(on_edge, "{p}");
            assert!(p.distance(avoid) >= 300.0, "{p} too close to {avoid}");
        }
    }
}
