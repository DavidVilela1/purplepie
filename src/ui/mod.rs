//! Clickable buttons for menus and HUDs.
//!
//! UI elements are ordinary entities in screen space
//! ([`ScreenSpace`]). This module adds the
//! interaction: a [`Button`] component that knows whether the mouse is over
//! it, holding it down, or has just clicked it. How a button looks is up to
//! the game (a `Quad`, a `Sprite`, a `Text`… on the same or other entities).
//!
//! Each frame, copy the mouse state out of the context, then run the system:
//!
//! ```
//! use purplepie::{ui, Context, Game};
//!
//! struct Menu;
//!
//! impl Game for Menu {
//!     fn update(&mut self, ctx: &mut Context<'_>) {
//!         let pointer = ui::Pointer::from_input(ctx.input());
//!         let viewport = ctx.viewport_size();
//!         ui::update_buttons(ctx.world_mut(), pointer, viewport);
//!         // … then read `Button::clicked()` on your buttons.
//!     }
//! }
//! ```
//!
//! Guide: section 9 (`docs/GUIDE.md`), with a complete button. Engine notes:
//! ADR-031 (topmost visible button wins; click = press and release over it).

use crate::ecs::hecs::Without;
use crate::ecs::{Entity, World};
use crate::input::{Input, MouseButton};
use crate::math::{Transform2D, Vec2};
use crate::render::{Hidden, Layer, ScreenSpace};

/// The mouse as UI sees it for one update: where the cursor is and what the
/// left button did. A plain `Copy` value, so it can be read from
/// [`Context::input`](crate::Context::input) before borrowing the world.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pointer {
    /// Cursor in window coordinates (logical pixels, top-left origin, +Y
    /// down), or `None` when it is outside the window.
    pub position: Option<Vec2>,
    /// The left button is held down.
    pub down: bool,
    /// The left button went down since the last update.
    pub just_pressed: bool,
    /// The left button went up since the last update.
    pub just_released: bool,
}

impl Pointer {
    /// The cursor and the left mouse button from `input`.
    pub fn from_input(input: &Input) -> Self {
        Self {
            position: input.cursor_position(),
            down: input.mouse_pressed(MouseButton::Left),
            just_pressed: input.mouse_just_pressed(MouseButton::Left),
            just_released: input.mouse_just_released(MouseButton::Left),
        }
    }
}

/// A clickable rectangle in screen space, `size` logical pixels, centred on
/// the entity's [`Transform2D`] (scaled by its scale; rotation is ignored).
///
/// The entity also needs a [`ScreenSpace`] component; buttons without one,
/// and [`Hidden`] buttons, never react. [`update_buttons`] sets the state:
///
/// - [`is_hovered`](Self::is_hovered): the cursor is over this button, and it
///   is the topmost button there (highest [`Layer`], then the most recently
///   spawned).
/// - [`is_pressed`](Self::is_pressed): the left button went down on this
///   button and is still held, with the cursor over it (for a "pushed" look).
/// - [`clicked`](Self::clicked): the left button went down **and** up on this
///   button. `true` for exactly one update. Pressing on the button and
///   releasing elsewhere (or the other way round) is not a click.
///
/// ```
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Color, Quad, ScreenSpace};
/// use purplepie::ui::Button;
///
/// let mut world = purplepie::ecs::World::new();
/// // A 160×40 button 20 px from the top-right corner, drawn as a quad.
/// world.spawn((
///     Transform2D::from_position(Vec2::new(-100.0, -40.0)),
///     ScreenSpace::TOP_RIGHT,
///     Button::new(Vec2::new(160.0, 40.0)),
///     Quad::new(Vec2::new(160.0, 40.0), Color::hex(0x3A86FF)),
/// ));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Button {
    /// Width and height of the clickable area in logical pixels.
    pub size: Vec2,
    hovered: bool,
    /// The left button went down on this button and has not been released.
    armed: bool,
    clicked: bool,
}

impl Button {
    /// A `size`-pixel button, idle.
    pub const fn new(size: Vec2) -> Self {
        Self {
            size,
            hovered: false,
            armed: false,
            clicked: false,
        }
    }

    /// The cursor is over this button (and no higher button covers it).
    pub const fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Being pushed: pressed on this button, still held, cursor still over it.
    pub const fn is_pressed(&self) -> bool {
        self.armed && self.hovered
    }

    /// Clicked in the last [`update_buttons`] call (press and release both on
    /// this button).
    pub const fn clicked(&self) -> bool {
        self.clicked
    }

    /// Whether `window` (window coordinates) is inside this button placed by
    /// `transform` in `screen` space in a `viewport`-sized window.
    fn contains(
        &self,
        transform: &Transform2D,
        screen: &ScreenSpace,
        window: Vec2,
        viewport: Vec2,
    ) -> bool {
        let local = screen.from_window(window, viewport) - transform.position;
        let half = self.size * transform.scale.abs() * 0.5;
        local.x.abs() <= half.x && local.y.abs() <= half.y
    }
}

/// Updates every [`Button`] in `world` from `pointer` (see [`Pointer::from_input`]);
/// `viewport` is [`Context::viewport_size`](crate::Context::viewport_size).
/// Call it once per update, before reading the buttons' state.
pub fn update_buttons(world: &mut World, pointer: Pointer, viewport: Vec2) {
    // The topmost visible button under the cursor: highest layer, then newest entity.
    let mut top: Option<((i32, u32), Entity)> = None;
    if let Some(position) = pointer.position {
        for (entity, transform, button, screen, layer) in world
            .query::<Without<
                (
                    Entity,
                    &Transform2D,
                    &Button,
                    &ScreenSpace,
                    Option<&Layer>,
                ),
                &Hidden,
            >>()
            .iter()
        {
            if button.contains(transform, screen, position, viewport) {
                let key = (layer.copied().unwrap_or_default().0, entity.id());
                if top.is_none_or(|(best, _)| key > best) {
                    top = Some((key, entity));
                }
            }
        }
    }
    let top = top.map(|(_, entity)| entity);
    for (entity, button, screen, hidden) in
        world.query_mut::<(Entity, &mut Button, Option<&ScreenSpace>, Option<&Hidden>)>()
    {
        button.clicked = false;
        if hidden.is_some() || screen.is_none() {
            button.hovered = false;
            button.armed = false;
            continue;
        }
        let hovered = top == Some(entity);
        if pointer.just_pressed && hovered {
            button.armed = true;
        }
        if pointer.just_released {
            button.clicked = button.armed && hovered;
            button.armed = false;
        } else if !pointer.down {
            // Released without an edge we saw (e.g. focus lost): disarm.
            button.armed = false;
        }
        button.hovered = hovered;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);

    /// A 100×40 button 60 px right of / 30 px below the top-left corner (its centre).
    fn world_with_button() -> (World, Entity) {
        let mut world = World::new();
        let button = world.spawn((
            Transform2D::from_position(Vec2::new(60.0, -30.0)),
            ScreenSpace::TOP_LEFT,
            Button::new(Vec2::new(100.0, 40.0)),
        ));
        (world, button)
    }

    fn at(x: f32, y: f32) -> Pointer {
        Pointer {
            position: Some(Vec2::new(x, y)),
            ..Pointer::default()
        }
    }

    fn state(world: &World, e: Entity) -> (bool, bool, bool) {
        let b = *world.get::<&Button>(e).expect("button");
        (b.is_hovered(), b.is_pressed(), b.clicked())
    }

    #[test]
    fn hover_follows_the_rectangle_edges() {
        let (mut world, b) = world_with_button();
        // The button covers window x 10..110, y 10..50.
        for (x, y, inside) in [
            (60.0, 30.0, true),
            (10.0, 10.0, true),
            (110.0, 50.0, true),
            (9.9, 30.0, false),
            (110.1, 30.0, false),
            (60.0, 50.1, false),
            (60.0, 9.9, false),
        ] {
            update_buttons(&mut world, at(x, y), VIEWPORT);
            assert_eq!(state(&world, b).0, inside, "({x}, {y})");
        }
        update_buttons(&mut world, Pointer::default(), VIEWPORT);
        assert!(!state(&world, b).0, "no cursor, no hover");
    }

    #[test]
    fn a_click_is_press_and_release_on_the_button_and_lasts_one_update() {
        let (mut world, b) = world_with_button();
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                just_pressed: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        assert_eq!(
            state(&world, b),
            (true, true, false),
            "pressed, not yet clicked"
        );
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                ..at(61.0, 31.0)
            },
            VIEWPORT,
        );
        assert_eq!(state(&world, b), (true, true, false), "still held");
        update_buttons(
            &mut world,
            Pointer {
                just_released: true,
                ..at(62.0, 32.0)
            },
            VIEWPORT,
        );
        assert_eq!(state(&world, b), (true, false, true), "clicked");
        update_buttons(&mut world, at(62.0, 32.0), VIEWPORT);
        assert_eq!(
            state(&world, b),
            (true, false, false),
            "only for one update"
        );
    }

    #[test]
    fn a_press_and_release_within_one_update_is_a_click() {
        let (mut world, b) = world_with_button();
        let quick = Pointer {
            just_pressed: true,
            just_released: true,
            ..at(60.0, 30.0)
        };
        update_buttons(&mut world, quick, VIEWPORT);
        assert!(state(&world, b).2);
    }

    #[test]
    fn dragging_off_or_onto_the_button_does_not_click() {
        let (mut world, b) = world_with_button();
        // Press inside, release outside.
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                just_pressed: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                ..at(300.0, 300.0)
            },
            VIEWPORT,
        );
        assert_eq!(
            state(&world, b),
            (false, false, false),
            "held but outside: not pushed"
        );
        update_buttons(
            &mut world,
            Pointer {
                just_released: true,
                ..at(300.0, 300.0)
            },
            VIEWPORT,
        );
        assert!(!state(&world, b).2);
        // Press outside, release inside.
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                just_pressed: true,
                ..at(300.0, 300.0)
            },
            VIEWPORT,
        );
        update_buttons(
            &mut world,
            Pointer {
                just_released: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        assert_eq!(state(&world, b), (true, false, false));
        // Press inside, leave, come back, release inside: still a click.
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                just_pressed: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                ..at(300.0, 300.0)
            },
            VIEWPORT,
        );
        update_buttons(
            &mut world,
            Pointer {
                just_released: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        assert!(state(&world, b).2);
    }

    #[test]
    fn a_release_we_never_saw_disarms_the_button() {
        let (mut world, b) = world_with_button();
        update_buttons(
            &mut world,
            Pointer {
                down: true,
                just_pressed: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        update_buttons(&mut world, at(60.0, 30.0), VIEWPORT); // up, without an edge
        update_buttons(
            &mut world,
            Pointer {
                just_released: true,
                ..at(60.0, 30.0)
            },
            VIEWPORT,
        );
        assert!(!state(&world, b).2);
    }

    #[test]
    fn only_the_topmost_button_reacts_where_buttons_overlap() {
        let (mut world, low) = world_with_button();
        let high = world.spawn((
            Transform2D::from_position(Vec2::new(60.0, -30.0)),
            ScreenSpace::TOP_LEFT,
            Button::new(Vec2::new(20.0, 20.0)),
            Layer(2),
        ));
        let click = |world: &mut World, x, y| {
            update_buttons(
                world,
                Pointer {
                    just_pressed: true,
                    just_released: true,
                    ..at(x, y)
                },
                VIEWPORT,
            );
        };
        click(&mut world, 60.0, 30.0);
        assert_eq!((state(&world, low).2, state(&world, high).2), (false, true));
        click(&mut world, 20.0, 30.0); // only the low button is here
        assert_eq!((state(&world, low).2, state(&world, high).2), (true, false));
        // Same layer: the newer entity wins.
        world.remove_one::<Layer>(high).expect("layer");
        click(&mut world, 60.0, 30.0);
        assert!(state(&world, high).2 && !state(&world, low).2);
    }

    #[test]
    fn hidden_or_unanchored_buttons_never_react() {
        let (mut world, b) = world_with_button();
        world.insert_one(b, Hidden).expect("hide");
        let click = Pointer {
            just_pressed: true,
            just_released: true,
            ..at(60.0, 30.0)
        };
        update_buttons(&mut world, click, VIEWPORT);
        assert_eq!(state(&world, b), (false, false, false));
        world.remove_one::<Hidden>(b).expect("show");
        world.remove_one::<ScreenSpace>(b).expect("unanchor");
        update_buttons(&mut world, click, VIEWPORT);
        assert_eq!(state(&world, b), (false, false, false));
    }

    #[test]
    fn anchors_scale_and_window_size_place_the_hit_area() {
        let mut world = World::new();
        // 40×20 button scaled ×2 (→ 80×40), 50 px left of / 30 px above the bottom-right corner.
        let b = world.spawn((
            Transform2D::from_position(Vec2::new(-50.0, 30.0)).with_scale(Vec2::splat(2.0)),
            ScreenSpace::BOTTOM_RIGHT,
            Button::new(Vec2::new(40.0, 20.0)),
        ));
        for viewport in [VIEWPORT, Vec2::new(1280.0, 720.0)] {
            let centre = viewport - Vec2::new(50.0, 30.0);
            for (offset, inside) in [
                (Vec2::ZERO, true),
                (Vec2::new(40.0, 20.0), true),
                (Vec2::new(40.5, 0.0), false),
                (Vec2::new(0.0, -20.5), false),
            ] {
                let p = centre + offset;
                update_buttons(&mut world, at(p.x, p.y), viewport);
                assert_eq!(state(&world, b).0, inside, "{viewport} {offset}");
            }
        }
    }

    #[test]
    fn pointer_reads_the_left_button_and_cursor() {
        let mut input = Input::default();
        input.set_cursor(Some(Vec2::new(3.0, 4.0)));
        input.mouse_down(MouseButton::Left);
        let p = Pointer::from_input(&input);
        assert_eq!(
            p,
            Pointer {
                position: Some(Vec2::new(3.0, 4.0)),
                down: true,
                just_pressed: true,
                just_released: false
            }
        );
        input.mouse_down(MouseButton::Right);
        assert!(Pointer::from_input(&input).down);
    }
}
