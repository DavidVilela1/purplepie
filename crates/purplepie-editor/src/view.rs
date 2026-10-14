//! The editor's view of the scene: what area the scene covers, and how the
//! camera fits, pans and zooms in the part of the window the panels leave
//! free. Plain functions, unit-tested without a window.

use purplepie::Context;
use purplepie::ecs::{Entity, World};
use purplepie::math::{Transform2D, Vec2};
use purplepie::render::{Camera2D, Hidden, Quad, ScreenSpace, Sprite, Text};

/// Margin around the scene when fitting, as a share of the free area.
const FIT_MARGIN: f32 = 0.9;
/// How much one wheel notch zooms.
pub(crate) const ZOOM_STEP: f32 = 1.1;
/// Zoom limits, so the view never collapses or explodes.
const MIN_ZOOM: f32 = 0.01;
const MAX_ZOOM: f32 = 100.0;

/// The world rectangle (min, max) covered by every visible, world-space quad,
/// sprite and text, or `None` for an empty scene. `text_bounds` gives a text's
/// local rectangle (from [`Context::measure_text`]).
pub(crate) fn scene_bounds(
    world: &World,
    text_bounds: impl Fn(&Text) -> Option<(Vec2, Vec2)>,
) -> Option<(Vec2, Vec2)> {
    let mut bounds: Option<(Vec2, Vec2)> = None;
    let mut add = |transform: &Transform2D, min: Vec2, max: Vec2| {
        let matrix = transform.to_mat4();
        for corner in [min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)] {
            let p = matrix.transform_point3(corner.extend(0.0));
            let p = Vec2::new(p.x, p.y);
            if !p.is_finite() {
                continue;
            }
            bounds = Some(match bounds {
                None => (p, p),
                Some((lo, hi)) => (lo.min(p), hi.max(p)),
            });
        }
    };
    for e in world.iter() {
        if e.has::<Hidden>() || e.has::<ScreenSpace>() {
            continue;
        }
        let Some(transform) = e.get::<&Transform2D>() else {
            continue;
        };
        if let Some(quad) = e.get::<&Quad>() {
            add(&transform, -quad.size.abs() * 0.5, quad.size.abs() * 0.5);
        }
        if let Some(sprite) = e.get::<&Sprite>() {
            add(
                &transform,
                -sprite.size.abs() * 0.5,
                sprite.size.abs() * 0.5,
            );
        }
        if let Some(text) = e.get::<&Text>()
            && let Some((min, max)) = text_bounds(&text)
        {
            add(&transform, min, max);
        }
    }
    bounds
}

/// The local rectangle (min, max) of what `entity` draws: its quad, else its
/// sprite, else its text (`text_bounds` as for [`scene_bounds`]).
fn drawn_rect(
    world: &World,
    entity: Entity,
    text_bounds: impl Fn(&Text) -> Option<(Vec2, Vec2)>,
) -> Option<(Vec2, Vec2)> {
    let e = world.entity(entity).ok()?;
    if let Some(quad) = e.get::<&Quad>() {
        return Some((-quad.size.abs() * 0.5, quad.size.abs() * 0.5));
    }
    if let Some(sprite) = e.get::<&Sprite>() {
        return Some((-sprite.size.abs() * 0.5, sprite.size.abs() * 0.5));
    }
    let text = e.get::<&Text>()?;
    text_bounds(&text)
}

/// The four corners, in window pixels (+Y down), of the rectangle `entity`
/// draws, turned and scaled by its transform and seen through `camera` (or
/// placed from its anchor when it is in screen space). `None` when it draws
/// nothing or has no transform.
pub(crate) fn outline(
    world: &World,
    entity: Entity,
    camera: &Camera2D,
    viewport: Vec2,
    text_bounds: impl Fn(&Text) -> Option<(Vec2, Vec2)>,
) -> Option<[Vec2; 4]> {
    let (min, max) = drawn_rect(world, entity, text_bounds)?;
    let transform = *world.get::<&Transform2D>(entity).ok()?;
    let screen = world.get::<&ScreenSpace>(entity).ok().map(|s| *s);
    let matrix = transform.to_mat4();
    let corners = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)].map(|corner| {
        let p = matrix.transform_point3(corner.extend(0.0));
        let p = Vec2::new(p.x, p.y);
        match &screen {
            Some(screen) => screen.to_window(p, viewport),
            None => camera.world_to_screen(p, viewport),
        }
    });
    corners.iter().all(|c| c.is_finite()).then_some(corners)
}

/// [`scene_bounds`] for the world in `ctx`, measuring texts with its fonts.
pub(crate) fn scene_bounds_in(ctx: &Context<'_>) -> Option<(Vec2, Vec2)> {
    scene_bounds(ctx.world(), |text| {
        ctx.measure_text(text).map(|m| m.bounds(text.anchor))
    })
}

/// A camera showing `bounds` (world) as large as fits, with a margin, centred
/// in the free area `free_min..free_max` (logical pixels, +Y down) of a
/// `viewport`-sized window. An empty scene or area keeps zoom 1 at the origin
/// of the free area.
pub(crate) fn fit(
    bounds: Option<(Vec2, Vec2)>,
    free_min: Vec2,
    free_max: Vec2,
    viewport: Vec2,
) -> Camera2D {
    let free = free_max - free_min;
    let (center, zoom) = match bounds {
        Some((min, max)) if free.x > 0.0 && free.y > 0.0 => {
            let size = (max - min).max(Vec2::splat(1.0));
            let zoom = ((free / size).min_element() * FIT_MARGIN).clamp(MIN_ZOOM, MAX_ZOOM);
            ((min + max) * 0.5, zoom)
        }
        _ => (Vec2::ZERO, 1.0),
    };
    // The window's centre shows the camera position; move it so `center`
    // lands in the middle of the free area instead.
    let offset = (free_min + free_max) * 0.5 - viewport * 0.5;
    Camera2D::new(center - Vec2::new(offset.x, -offset.y) / zoom, zoom)
}

/// The camera moved so the scene follows a drag of `delta` logical pixels
/// (+Y down).
pub(crate) fn pan(camera: Camera2D, delta: Vec2) -> Camera2D {
    let zoom = camera.effective_zoom();
    Camera2D::new(camera.position - Vec2::new(delta.x, -delta.y) / zoom, zoom)
}

/// The camera zoomed by `factor` around the screen point `at` (logical
/// pixels), which keeps showing the same world point.
pub(crate) fn zoom_at(camera: Camera2D, viewport: Vec2, at: Vec2, factor: f32) -> Camera2D {
    let before = camera.screen_to_world(at, viewport);
    let zoom = (camera.effective_zoom() * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    let zoomed = Camera2D::new(camera.position, zoom);
    let after = zoomed.screen_to_world(at, viewport);
    Camera2D::new(camera.position + (before - after), zoom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use purplepie::render::Color;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).abs().max_element() < 1e-3
    }

    #[test]
    fn bounds_cover_visible_world_drawables_with_their_transforms() {
        let mut world = World::new();
        assert_eq!(scene_bounds(&world, |_| None), None);
        world.spawn((
            Transform2D::from_position(Vec2::new(10.0, 0.0)),
            Quad::new(Vec2::new(4.0, 2.0), Color::WHITE),
        ));
        // Turned 90°: 2 wide and 4 tall around (-10, 5).
        world.spawn((
            Transform2D::from_position(Vec2::new(-10.0, 5.0))
                .with_rotation(std::f32::consts::FRAC_PI_2),
            Quad::new(Vec2::new(4.0, 2.0), Color::WHITE),
        ));
        // Hidden, screen-space and transform-less entities do not count.
        world.spawn((
            Transform2D::default(),
            Quad::new(Vec2::splat(1000.0), Color::WHITE),
            Hidden,
        ));
        world.spawn((
            Transform2D::default(),
            Quad::new(Vec2::splat(1000.0), Color::WHITE),
            ScreenSpace::CENTER,
        ));
        world.spawn((Quad::new(Vec2::splat(1000.0), Color::WHITE),));
        let (min, max) = scene_bounds(&world, |_| None).expect("not empty");
        assert!(close(min, Vec2::new(-11.0, -1.0)), "{min}");
        assert!(close(max, Vec2::new(12.0, 7.0)), "{max}");
    }

    #[test]
    fn outlines_follow_the_transform_the_camera_and_screen_space() {
        let viewport = Vec2::new(800.0, 600.0);
        let camera = Camera2D::new(Vec2::new(10.0, 0.0), 2.0);
        let mut world = World::new();
        let quad = world.spawn((
            Transform2D::from_position(Vec2::new(10.0, 0.0))
                .with_rotation(std::f32::consts::FRAC_PI_2),
            Quad::new(Vec2::new(4.0, 2.0), Color::WHITE),
        ));
        let corners = outline(&world, quad, &camera, viewport, |_| None).expect("drawn");
        // Turned 90°: 2 wide, 4 tall around the window centre, at zoom 2.
        let xs: Vec<f32> = corners.iter().map(|c| c.x).collect();
        let ys: Vec<f32> = corners.iter().map(|c| c.y).collect();
        let span = |v: &[f32]| {
            v.iter().copied().fold(f32::MIN, f32::max) - v.iter().copied().fold(f32::MAX, f32::min)
        };
        assert!((span(&xs) - 4.0).abs() < 1e-3 && (span(&ys) - 8.0).abs() < 1e-3);
        assert!(corners.iter().all(|c| (c.x - 400.0).abs() <= 2.001));

        // A HUD quad 20 px right of and 10 px below the top-left corner.
        let hud = world.spawn((
            Transform2D::from_position(Vec2::new(20.0, -10.0)),
            Quad::new(Vec2::splat(4.0), Color::WHITE),
            ScreenSpace::TOP_LEFT,
        ));
        let corners = outline(&world, hud, &camera, viewport, |_| None).expect("drawn");
        assert!(close(corners[0], Vec2::new(18.0, 12.0)), "{:?}", corners);
        assert!(close(corners[2], Vec2::new(22.0, 8.0)), "{:?}", corners);

        let bare = world.spawn((Transform2D::default(),));
        assert!(outline(&world, bare, &camera, viewport, |_| None).is_none());
        let text = world.spawn((Transform2D::default(), 5_u8));
        assert!(outline(&world, text, &camera, viewport, |_| None).is_none());
    }

    #[test]
    fn fitting_centres_the_scene_in_the_free_area_with_a_margin() {
        let viewport = Vec2::new(1000.0, 600.0);
        // A 250-pixel panel on the left and a 20-pixel status line below.
        let (free_min, free_max) = (Vec2::new(250.0, 0.0), Vec2::new(1000.0, 580.0));
        let bounds = Some((Vec2::new(-100.0, -50.0), Vec2::new(300.0, 150.0)));
        let camera = fit(bounds, free_min, free_max, viewport);
        // 400 × 200 world units into 750 × 580 pixels: width limits, ×0.9.
        assert!((camera.zoom - 750.0 / 400.0 * 0.9).abs() < 1e-4);
        let centre = camera.world_to_screen(Vec2::new(100.0, 50.0), viewport);
        assert!(close(centre, Vec2::new(625.0, 290.0)), "{centre}");
        let left = camera.world_to_screen(Vec2::new(-100.0, 50.0), viewport);
        assert!(left.x > free_min.x, "the scene starts right of the panel");

        let empty = fit(None, free_min, free_max, viewport);
        assert_eq!(empty.zoom, 1.0);
        let origin = empty.world_to_screen(Vec2::ZERO, viewport);
        assert!(close(origin, Vec2::new(625.0, 290.0)), "{origin}");
    }

    #[test]
    fn panning_follows_the_drag_and_zooming_keeps_the_point_under_the_cursor() {
        let viewport = Vec2::new(800.0, 600.0);
        let camera = Camera2D::new(Vec2::new(5.0, 5.0), 2.0);
        let grabbed = camera.screen_to_world(Vec2::new(100.0, 100.0), viewport);
        let moved = pan(camera, Vec2::new(30.0, -40.0));
        let now = moved.world_to_screen(grabbed, viewport);
        assert!(close(now, Vec2::new(130.0, 60.0)), "{now}");

        let at = Vec2::new(700.0, 120.0);
        let under = camera.screen_to_world(at, viewport);
        let zoomed = zoom_at(camera, viewport, at, ZOOM_STEP.powi(3));
        assert!((zoomed.zoom - 2.0 * ZOOM_STEP.powi(3)).abs() < 1e-4);
        assert!(close(zoomed.screen_to_world(at, viewport), under));
        assert_eq!(zoom_at(camera, viewport, at, 1e9).zoom, MAX_ZOOM);
        assert_eq!(zoom_at(camera, viewport, at, 1e-9).zoom, MIN_ZOOM);
    }
}
