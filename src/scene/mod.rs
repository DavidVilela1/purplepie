//! Scene files: the engine's own components saved to and loaded from text
//! (ADR-035).
//!
//! Crate-private. Games use [`Context::save_scene`](crate::Context::save_scene)
//! and [`Context::load_scene`](crate::Context::load_scene). A scene file is
//! [RON](https://docs.rs/ron) text written from plain mirror types (the `*File`
//! structs below), never from the components themselves, so `serde` stays out
//! of the public API and the file layout can stay stable while components
//! change. Assets are referenced by path relative to the asset root, never by
//! the per-run `TextureId`/`FontId`.
//!
//! Version 1 covers `Transform2D`, `Quad`, `Sprite`, `Text`, `Layer`, `Hidden`,
//! `ScreenSpace` (PP-026a), `SpriteAnimation` with its playback position,
//! `Velocity` and `ui::Button` (PP-026b) on every entity that has a `Quad`,
//! `Sprite`, `Text`, `SpriteAnimation` or `Button`. Fields added later are
//! optional, so every version-1 file keeps loading. The game's own components
//! are not saved yet.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ecs::{Entity, Velocity, World};
use crate::error::BoxError;
use crate::math::{Transform2D, Vec2};
use crate::render::{
    AnimationMode, Color, FontId, Fonts, Hidden, HorizontalAnchor, Layer, Quad, ScreenAnchor,
    ScreenSpace, Sprite, SpriteAnimation, SpriteGrid, Text, TextAnchor, TextureFilter, TextureId,
    TextureOptions, TextureRegion, Textures, VerticalAnchor,
};
use crate::ui::Button;

/// The scene format version this build writes and reads.
pub(crate) const VERSION: u32 = 1;

/// A whole scene file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SceneFile {
    pub(crate) version: u32,
    pub(crate) entities: Vec<EntityFile>,
}

/// Only the version, read first so a newer file gets a clear message.
#[derive(Deserialize)]
struct VersionOnly {
    version: u32,
}

/// One entity: each engine component it has. Absent fields are absent
/// components.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub(crate) struct EntityFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    transform: Option<TransformFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quad: Option<QuadFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprite: Option<SpriteFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    layer: Option<i32>,
    #[serde(skip_serializing_if = "is_false")]
    hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen_space: Option<AnchorFile>,
    // PP-026b: optional, so PP-026a files still load.
    #[serde(skip_serializing_if = "Option::is_none")]
    animation: Option<AnimationFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    velocity: Option<Vec2File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    button: Option<ButtonFile>,
}

/// Whether an entity is saved: it has something the engine draws or makes
/// interactive. Entities with only a transform, velocity or game components
/// are usually game bookkeeping that a scene cannot rebuild yet.
fn is_saved(e: &hecs::EntityRef<'_>) -> bool {
    e.has::<Quad>()
        || e.has::<Sprite>()
        || e.has::<Text>()
        || e.has::<SpriteAnimation>()
        || e.has::<Button>()
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// `(x, y)`.
type Vec2File = (f32, f32);
/// `(r, g, b, a)`: sRGB, straight alpha, like [`Color`].
type ColorFile = (f32, f32, f32, f32);

const WHITE: ColorFile = (1.0, 1.0, 1.0, 1.0);

fn vec2(v: Vec2File) -> Vec2 {
    Vec2::new(v.0, v.1)
}

fn vec2_file(v: Vec2) -> Vec2File {
    (v.x, v.y)
}

fn color(c: ColorFile) -> Color {
    Color::rgba(c.0, c.1, c.2, c.3)
}

fn color_file(c: Color) -> ColorFile {
    (c.r, c.g, c.b, c.a)
}

fn is_white(c: &ColorFile) -> bool {
    *c == WHITE
}

fn white() -> ColorFile {
    WHITE
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransformFile {
    #[serde(default)]
    position: Vec2File,
    #[serde(default, skip_serializing_if = "is_zero")]
    rotation: f32,
    #[serde(default = "unit_scale", skip_serializing_if = "is_unit_scale")]
    scale: Vec2File,
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

fn unit_scale() -> Vec2File {
    (1.0, 1.0)
}

fn is_unit_scale(v: &Vec2File) -> bool {
    *v == (1.0, 1.0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QuadFile {
    size: Vec2File,
    color: ColorFile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpriteFile {
    /// Asset path, relative to the asset root (or absolute).
    texture: String,
    #[serde(default, skip_serializing_if = "is_nearest")]
    filter: FilterFile,
    size: Vec2File,
    #[serde(default = "white", skip_serializing_if = "is_white")]
    tint: ColorFile,
    /// `(x, y, width, height)` in texels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    region: Option<(u32, u32, u32, u32)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum FilterFile {
    #[default]
    Nearest,
    Linear,
}

fn is_nearest(f: &FilterFile) -> bool {
    *f == FilterFile::Nearest
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TextFile {
    content: String,
    /// Asset path, relative to the asset root (or absolute).
    font: String,
    size: f32,
    #[serde(default = "white", skip_serializing_if = "is_white")]
    color: ColorFile,
    #[serde(default, skip_serializing_if = "is_default_anchor")]
    anchor: (HorizontalFile, VerticalFile),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum HorizontalFile {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum VerticalFile {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

fn is_default_anchor(a: &(HorizontalFile, VerticalFile)) -> bool {
    *a == Default::default()
}

/// [`SpriteAnimation`]: what to play and, for editor snapshots, where
/// playback is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnimationFile {
    grid: GridFile,
    first: u32,
    last: u32,
    fps: f32,
    #[serde(default, skip_serializing_if = "is_loop")]
    mode: ModeFile,
    /// Frames played since `first`.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    step: u32,
    /// Seconds spent on the current frame (`f64`, as the animation keeps it).
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    time_in_frame: f64,
    #[serde(default, skip_serializing_if = "is_false")]
    finished: bool,
}

/// [`SpriteGrid`]: `cell` is `(width, height)` in texels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GridFile {
    cell: (u32, u32),
    columns: u32,
    rows: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    spacing: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    margin: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum ModeFile {
    #[default]
    Loop,
    Once,
}

fn is_loop(m: &ModeFile) -> bool {
    *m == ModeFile::Loop
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

/// [`Button`]: only its size; hover/press/click state is per run and starts idle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ButtonFile {
    size: Vec2File,
}

fn animation_file(a: &SpriteAnimation) -> AnimationFile {
    let (step, time_in_frame, finished) = a.playback();
    AnimationFile {
        grid: GridFile {
            cell: (a.grid.cell_width, a.grid.cell_height),
            columns: a.grid.columns,
            rows: a.grid.rows,
            spacing: a.grid.spacing,
            margin: a.grid.margin,
        },
        first: a.first,
        last: a.last,
        fps: a.fps,
        mode: match a.mode {
            AnimationMode::Loop => ModeFile::Loop,
            AnimationMode::Once => ModeFile::Once,
        },
        step,
        time_in_frame,
        finished,
    }
}

fn animation(a: AnimationFile) -> SpriteAnimation {
    let grid = SpriteGrid::new(a.grid.cell.0, a.grid.cell.1, a.grid.columns, a.grid.rows)
        .with_spacing(a.grid.spacing)
        .with_margin(a.grid.margin);
    let mut animation = SpriteAnimation::new(grid, a.first, a.last, a.fps);
    if a.mode == ModeFile::Once {
        animation = animation.once();
    }
    animation.with_playback(a.step, a.time_in_frame, a.finished)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum AnchorFile {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl From<ScreenAnchor> for AnchorFile {
    fn from(a: ScreenAnchor) -> Self {
        match a {
            ScreenAnchor::TopLeft => Self::TopLeft,
            ScreenAnchor::Top => Self::Top,
            ScreenAnchor::TopRight => Self::TopRight,
            ScreenAnchor::Left => Self::Left,
            ScreenAnchor::Center => Self::Center,
            ScreenAnchor::Right => Self::Right,
            ScreenAnchor::BottomLeft => Self::BottomLeft,
            ScreenAnchor::Bottom => Self::Bottom,
            ScreenAnchor::BottomRight => Self::BottomRight,
        }
    }
}

impl From<AnchorFile> for ScreenAnchor {
    fn from(a: AnchorFile) -> Self {
        match a {
            AnchorFile::TopLeft => Self::TopLeft,
            AnchorFile::Top => Self::Top,
            AnchorFile::TopRight => Self::TopRight,
            AnchorFile::Left => Self::Left,
            AnchorFile::Center => Self::Center,
            AnchorFile::Right => Self::Right,
            AnchorFile::BottomLeft => Self::BottomLeft,
            AnchorFile::Bottom => Self::Bottom,
            AnchorFile::BottomRight => Self::BottomRight,
        }
    }
}

fn anchor_file(a: TextAnchor) -> (HorizontalFile, VerticalFile) {
    let h = match a.horizontal {
        HorizontalAnchor::Left => HorizontalFile::Left,
        HorizontalAnchor::Center => HorizontalFile::Center,
        HorizontalAnchor::Right => HorizontalFile::Right,
    };
    let v = match a.vertical {
        VerticalAnchor::Baseline => VerticalFile::Baseline,
        VerticalAnchor::Top => VerticalFile::Top,
        VerticalAnchor::Middle => VerticalFile::Middle,
        VerticalAnchor::Bottom => VerticalFile::Bottom,
    };
    (h, v)
}

fn text_anchor((h, v): (HorizontalFile, VerticalFile)) -> TextAnchor {
    let h = match h {
        HorizontalFile::Left => HorizontalAnchor::Left,
        HorizontalFile::Center => HorizontalAnchor::Center,
        HorizontalFile::Right => HorizontalAnchor::Right,
    };
    let v = match v {
        VerticalFile::Baseline => VerticalAnchor::Baseline,
        VerticalFile::Top => VerticalAnchor::Top,
        VerticalFile::Middle => VerticalAnchor::Middle,
        VerticalFile::Bottom => VerticalAnchor::Bottom,
    };
    TextAnchor::new(h, v)
}

/// How an asset path is written: relative to `root` with `/` separators when
/// it lies inside the asset root, otherwise as it is (absolute).
fn asset_reference(source: &Path, root: Option<&Path>) -> Result<String, BoxError> {
    let relative = root.and_then(|root| source.strip_prefix(root).ok());
    let path = relative.unwrap_or(source);
    let parts: Option<Vec<&str>> = if relative.is_some() {
        path.components().map(|c| c.as_os_str().to_str()).collect()
    } else {
        None
    };
    match parts {
        Some(parts) => Ok(parts.join("/")),
        None => path
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("asset path {} is not valid UTF-8", path.display()).into()),
    }
}

/// Builds the scene file for every saved entity in `world` (see [`is_saved`]),
/// in entity order (normally spawn order).
pub(crate) fn capture(
    world: &World,
    textures: &Textures,
    fonts: &Fonts,
    asset_root: Option<&Path>,
) -> Result<SceneFile, BoxError> {
    let mut entities: Vec<_> = world.iter().filter(is_saved).collect();
    entities.sort_by_key(|e| e.entity().id());
    let mut out = Vec::with_capacity(entities.len());
    for e in entities {
        let transform = e.get::<&Transform2D>().map(|t| TransformFile {
            position: vec2_file(t.position),
            rotation: t.rotation,
            scale: vec2_file(t.scale),
        });
        let quad = e.get::<&Quad>().map(|q| QuadFile {
            size: vec2_file(q.size),
            color: color_file(q.color),
        });
        let sprite = match e.get::<&Sprite>() {
            Some(s) => {
                let data = textures
                    .get(s.texture)
                    .ok_or("a sprite refers to a texture that is not loaded")?;
                Some(SpriteFile {
                    texture: asset_reference(&data.source, asset_root)?,
                    filter: match data.filter {
                        TextureFilter::Nearest => FilterFile::Nearest,
                        TextureFilter::Linear => FilterFile::Linear,
                    },
                    size: vec2_file(s.size),
                    tint: color_file(s.tint),
                    region: s.region.map(|r| (r.x, r.y, r.width, r.height)),
                })
            }
            None => None,
        };
        let text = match e.get::<&Text>() {
            Some(t) => {
                let data = fonts
                    .get(t.font)
                    .ok_or("a text refers to a font that is not loaded")?;
                Some(TextFile {
                    content: t.content.clone(),
                    font: asset_reference(&data.source, asset_root)?,
                    size: t.size,
                    color: color_file(t.color),
                    anchor: anchor_file(t.anchor),
                })
            }
            None => None,
        };
        out.push(EntityFile {
            transform,
            quad,
            sprite,
            text,
            layer: e.get::<&Layer>().map(|l| l.0),
            hidden: e.has::<Hidden>(),
            screen_space: e.get::<&ScreenSpace>().map(|s| s.anchor.into()),
            animation: e.get::<&SpriteAnimation>().map(|a| animation_file(&a)),
            velocity: e.get::<&Velocity>().map(|v| vec2_file(v.0)),
            button: e.get::<&Button>().map(|b| ButtonFile {
                size: vec2_file(b.size),
            }),
        });
    }
    Ok(SceneFile {
        version: VERSION,
        entities: out,
    })
}

fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// Writes `scene` as RON text.
pub(crate) fn to_text(scene: &SceneFile) -> Result<String, BoxError> {
    // `ron` defaults to "\r\n" on Windows; one line ending everywhere keeps
    // scene files identical across platforms (and diff-friendly in git).
    let pretty = ron::ser::PrettyConfig::new()
        .new_line("\n")
        .indentor("    ")
        .struct_names(false)
        .extensions(ron::extensions::Extensions::IMPLICIT_SOME);
    let body = ron_options().to_string_pretty(scene, pretty)?;
    Ok(format!(
        "// PurplePie scene (ADR-035). Asset paths are relative to the asset root.\n{body}\n"
    ))
}

/// Parses RON text into a scene, checking the version first.
pub(crate) fn from_text(text: &str) -> Result<SceneFile, BoxError> {
    let options = ron_options();
    match options.from_str::<VersionOnly>(text) {
        Ok(v) if v.version != VERSION => {
            return Err(format!(
                "scene version {} is not supported (this PurplePie reads version {VERSION})",
                v.version
            )
            .into());
        }
        Ok(_) => {}
        Err(e) => return Err(format!("not a PurplePie scene file: {e}").into()),
    }
    Ok(options.from_str::<SceneFile>(text)?)
}

/// The assets a scene needs, resolved before anything is spawned: for each
/// entity in file order, its sprite's texture and its text's font.
pub(crate) struct Resolved {
    per_entity: Vec<(Option<TextureId>, Option<FontId>)>,
}

/// Loads every asset `scene` refers to, in entity order, with the given
/// loaders (which resolve paths against the asset root). Fails on the first
/// asset that cannot be loaded; nothing is spawned yet.
pub(crate) fn resolve_assets(
    scene: &SceneFile,
    mut load_texture: impl FnMut(&Path, TextureOptions) -> crate::Result<TextureId>,
    mut load_font: impl FnMut(&Path) -> crate::Result<FontId>,
) -> crate::Result<Resolved> {
    let mut per_entity = Vec::with_capacity(scene.entities.len());
    for e in &scene.entities {
        let texture = match &e.sprite {
            Some(s) => {
                let options = TextureOptions::default().with_filter(match s.filter {
                    FilterFile::Nearest => TextureFilter::Nearest,
                    FilterFile::Linear => TextureFilter::Linear,
                });
                Some(load_texture(&PathBuf::from(&s.texture), options)?)
            }
            None => None,
        };
        let font = match &e.text {
            Some(t) => Some(load_font(&PathBuf::from(&t.font))?),
            None => None,
        };
        per_entity.push((texture, font));
    }
    Ok(Resolved { per_entity })
}

/// Spawns the scene's entities into `world`, in file order, using the assets
/// from [`resolve_assets`]. Returns the new entities.
pub(crate) fn spawn(scene: SceneFile, assets: Resolved, world: &mut World) -> Vec<Entity> {
    let mut spawned = Vec::with_capacity(scene.entities.len());
    for (e, (texture, font)) in scene.entities.into_iter().zip(assets.per_entity) {
        let entity = world.spawn(());
        if let Some(t) = e.transform {
            let transform = Transform2D {
                position: vec2(t.position),
                rotation: t.rotation,
                scale: vec2(t.scale),
            };
            insert(world, entity, transform);
        }
        if let Some(q) = e.quad {
            insert(world, entity, Quad::new(vec2(q.size), color(q.color)));
        }
        if let (Some(s), Some(texture)) = (e.sprite, texture) {
            let mut sprite = Sprite::new(texture, vec2(s.size)).with_tint(color(s.tint));
            if let Some((x, y, w, h)) = s.region {
                sprite = sprite.with_region(TextureRegion::new(x, y, w, h));
            }
            insert(world, entity, sprite);
        }
        if let (Some(t), Some(font)) = (e.text, font) {
            let text = Text::new(t.content, font, t.size)
                .with_color(color(t.color))
                .with_anchor(text_anchor(t.anchor));
            insert(world, entity, text);
        }
        if let Some(layer) = e.layer {
            insert(world, entity, Layer(layer));
        }
        if e.hidden {
            insert(world, entity, Hidden);
        }
        if let Some(anchor) = e.screen_space {
            insert(world, entity, ScreenSpace::new(anchor.into()));
        }
        if let Some(a) = e.animation {
            insert(world, entity, animation(a));
        }
        if let Some(v) = e.velocity {
            insert(world, entity, Velocity(vec2(v)));
        }
        if let Some(b) = e.button {
            insert(world, entity, Button::new(vec2(b.size)));
        }
        spawned.push(entity);
    }
    spawned
}

/// Adds one component to an entity spawned just before (it always exists).
fn insert(world: &mut World, entity: Entity, component: impl hecs::Component) {
    if world.insert_one(entity, component).is_err() {
        log::warn!("scene: entity {entity:?} vanished while loading");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    /// Loads like `Context` does: relative paths against `root()`.
    fn load(
        scene: &SceneFile,
        textures: &mut Textures,
        fonts: &mut Fonts,
        world: &mut World,
    ) -> crate::Result<Vec<Entity>> {
        let root = root();
        let resolved = resolve_assets(
            scene,
            |p, o| textures.load(&root.join(p), o),
            |p| fonts.load(&root.join(p)),
        )?;
        Ok(spawn(scene.clone(), resolved, world))
    }

    /// A world using every saved component and option.
    fn sample() -> (World, Textures, Fonts) {
        let root = root();
        let mut textures = Textures::default();
        let mut fonts = Fonts::default();
        let sheet = textures
            .load(
                &root.join("textures/sandbox_sheet.png"),
                TextureOptions::LINEAR,
            )
            .expect("sheet");
        let quadrants = textures
            .load(
                &root.join("textures/sandbox_quadrants.png"),
                TextureOptions::NEAREST,
            )
            .expect("quadrants");
        let font = fonts
            .load(&root.join("fonts/Poppins-Regular.ttf"))
            .expect("font");
        let mut world = World::new();
        world.spawn((
            Transform2D::from_position(Vec2::new(1.5, -2.0)),
            Quad::new(Vec2::new(10.0, 20.0), Color::hex(0xFFB000)),
        ));
        world.spawn((Transform2D::default(),)); // no drawable: not saved
        world.spawn((
            Transform2D::from_position(Vec2::new(-7.25, 3.0))
                .with_rotation(0.25)
                .with_scale(Vec2::new(-1.0, 2.0)),
            Sprite::new(sheet, Vec2::splat(32.0))
                .with_region(TextureRegion::new(8, 0, 8, 8))
                .with_tint(Color::rgba(0.5, 1.0, 1.0, 0.6)),
            Layer(-3),
            Hidden,
        ));
        world.spawn((Sprite::new(quadrants, Vec2::new(96.0, 64.0)),));
        world.spawn((
            Transform2D::from_position(Vec2::new(20.0, -30.0)),
            Text::new("Hi \"there\"\nline two", font, 20.0)
                .with_color(Color::hex(0x1D3557))
                .with_anchor(TextAnchor::new(
                    HorizontalAnchor::Right,
                    VerticalAnchor::Middle,
                )),
            Quad::new(Vec2::new(4.0, 4.0), Color::BLACK),
            ScreenSpace::BOTTOM_RIGHT,
            Layer(7),
        ));
        // PP-026b: an animation caught mid-frame, a finished one-shot without a
        // sprite, a moving quad, a button, and a moving point that is not saved.
        let grid = SpriteGrid::new(8, 8, 4, 2).with_spacing(1).with_margin(2);
        let mut walking = SpriteAnimation::new(grid, 6, 1, 7.5);
        walking.advance(0.7);
        assert!(walking.playback().0 > 0 && walking.playback().1 > 0.0);
        world.spawn((
            Transform2D::from_position(Vec2::new(5.0, 5.0)),
            Sprite::new(sheet, Vec2::splat(16.0)),
            walking,
        ));
        let mut burst = SpriteAnimation::new(SpriteGrid::new(4, 4, 2, 2), 0, 3, 30.0).once();
        burst.advance(1.0);
        assert!(burst.is_finished());
        world.spawn((burst,));
        world.spawn((
            Transform2D::default(),
            Quad::new(Vec2::splat(3.0), Color::WHITE),
            Velocity(Vec2::new(-12.5, 40.0)),
        ));
        world.spawn((
            Transform2D::from_position(Vec2::new(-100.0, -40.0)),
            Quad::new(Vec2::new(160.0, 40.0), Color::hex(0x3A86FF)),
            Button::new(Vec2::new(160.0, 40.0)),
            ScreenSpace::TOP_RIGHT,
        ));
        world.spawn((Transform2D::default(), Velocity(Vec2::ONE))); // not saved
        (world, textures, fonts)
    }

    #[test]
    fn every_saved_component_survives_a_round_trip_through_text() {
        let (world, textures, fonts) = sample();
        let scene = capture(&world, &textures, &fonts, Some(&root())).expect("capture");
        assert_eq!(
            scene.entities.len(),
            8,
            "the transform-only and transform+velocity entities are skipped"
        );
        let text = to_text(&scene).expect("write");
        assert!(
            !text.contains('\r'),
            "\\n line ends on every platform, Windows included"
        );
        let parsed = from_text(&text).expect("read");
        assert_eq!(parsed, scene);

        let (mut textures2, mut fonts2) = (Textures::default(), Fonts::default());
        let mut world2 = World::new();
        let spawned = load(&parsed, &mut textures2, &mut fonts2, &mut world2).expect("load");
        assert_eq!(spawned.len(), 8);
        // Same components, entity by entity in file order (= spawn order).
        let mut originals: Vec<Entity> =
            world.iter().filter(is_saved).map(|e| e.entity()).collect();
        originals.sort_by_key(|e| e.id());
        for (&a, &b) in originals.iter().zip(&spawned) {
            let (ea, eb) = (world.entity(a).expect("a"), world2.entity(b).expect("b"));
            assert_eq!(
                ea.get::<&Transform2D>().map(|t| *t),
                eb.get::<&Transform2D>().map(|t| *t)
            );
            assert_eq!(ea.get::<&Quad>().map(|q| *q), eb.get::<&Quad>().map(|q| *q));
            assert_eq!(
                ea.get::<&Layer>().map(|l| *l),
                eb.get::<&Layer>().map(|l| *l)
            );
            assert_eq!(ea.has::<Hidden>(), eb.has::<Hidden>());
            // Whole-value equality, private playback state included.
            assert_eq!(
                ea.get::<&SpriteAnimation>().map(|a| *a),
                eb.get::<&SpriteAnimation>().map(|a| *a)
            );
            assert_eq!(
                ea.get::<&Velocity>().map(|v| *v),
                eb.get::<&Velocity>().map(|v| *v)
            );
            assert_eq!(
                ea.get::<&Button>().map(|b| *b),
                eb.get::<&Button>().map(|b| *b)
            );
            assert_eq!(
                ea.get::<&ScreenSpace>().map(|s| *s),
                eb.get::<&ScreenSpace>().map(|s| *s)
            );
            match (ea.get::<&Sprite>(), eb.get::<&Sprite>()) {
                (Some(sa), Some(sb)) => {
                    let (ta, tb) = (
                        textures.get(sa.texture).expect("ta"),
                        textures2.get(sb.texture).expect("tb"),
                    );
                    assert_eq!((&ta.source, ta.filter), (&tb.source, tb.filter));
                    assert_eq!(ta.pixels, tb.pixels);
                    assert_eq!((sa.size, sa.tint, sa.region), (sb.size, sb.tint, sb.region));
                }
                (None, None) => {}
                _ => panic!("sprite on one side only"),
            }
            match (ea.get::<&Text>(), eb.get::<&Text>()) {
                (Some(xa), Some(xb)) => {
                    assert_eq!(
                        fonts.get(xa.font).expect("fa").source,
                        fonts2.get(xb.font).expect("fb").source
                    );
                    assert_eq!(
                        (&xa.content, xa.size, xa.color, xa.anchor),
                        (&xb.content, xb.size, xb.color, xb.anchor)
                    );
                }
                (None, None) => {}
                _ => panic!("text on one side only"),
            }
        }
        // Saving the loaded world again gives the same text.
        let again = capture(&world2, &textures2, &fonts2, Some(&root())).expect("again");
        assert_eq!(to_text(&again).expect("write"), text);
    }

    #[test]
    fn a_restored_animation_continues_exactly_like_the_original() {
        let (world, textures, fonts) = sample();
        let scene = capture(&world, &textures, &fonts, Some(&root())).expect("capture");
        let parsed = from_text(&to_text(&scene).expect("write")).expect("read");
        let (mut t2, mut f2, mut world2) = (Textures::default(), Fonts::default(), World::new());
        load(&parsed, &mut t2, &mut f2, &mut world2).expect("load");
        let frames = |world: &mut World| {
            let mut seen = Vec::new();
            for _ in 0..40 {
                crate::render::advance_animations(world, 1.0 / 60.0);
                let mut frames: Vec<(u32, bool)> = world
                    .query::<&SpriteAnimation>()
                    .iter()
                    .map(|a| (a.frame(), a.is_finished()))
                    .collect();
                frames.sort_unstable();
                seen.push(frames);
            }
            seen
        };
        let mut original = World::new();
        for a in world.query::<&SpriteAnimation>().iter() {
            original.spawn((*a,));
        }
        assert_eq!(frames(&mut original), frames(&mut world2));
    }

    #[test]
    fn files_from_part_1a_still_load_and_save_back_unchanged() {
        // Written by PP-026a's `save_scene`: a version-1 file before animation,
        // velocity and buttons existed. Kept forever as a compatibility check.
        let text = include_str!("fixtures/demo_v1_pp026a.ron").replace("\r\n", "\n");
        let scene = from_text(&text).expect("parse");
        let (mut textures, mut fonts, mut world) =
            (Textures::default(), Fonts::default(), World::new());
        let spawned = load(&scene, &mut textures, &mut fonts, &mut world).expect("load");
        assert_eq!(spawned.len(), 14);
        let again = capture(&world, &textures, &fonts, Some(&root())).expect("capture");
        assert_eq!(to_text(&again).expect("write"), text);
    }

    #[test]
    fn asset_paths_are_relative_to_the_root_with_forward_slashes() {
        let root = root();
        assert_eq!(
            asset_reference(&root.join("textures").join("a.png"), Some(&root)).expect("ref"),
            "textures/a.png"
        );
        let outside = std::env::temp_dir().join("elsewhere.png");
        assert_eq!(
            asset_reference(&outside, Some(&root)).expect("ref"),
            outside.to_str().expect("utf-8")
        );
        assert_eq!(
            asset_reference(&outside, None).expect("ref"),
            outside.to_str().expect("utf-8")
        );
    }

    #[test]
    fn hand_written_scenes_use_defaults_for_omitted_fields() {
        let text = r#"(
            version: 1,
            entities: [
                (quad: (size: (2.0, 3.0), color: (1.0, 0.0, 0.0, 1.0))),
                (transform: (), sprite: (texture: "textures/sandbox_sheet.png", size: (8.0, 8.0))),
                (),
            ],
        )"#;
        let scene = from_text(text).expect("parse");
        assert_eq!(
            from_text(&text.replace('\n', "\r\n")).expect("CRLF"),
            scene,
            "Windows line ends read the same"
        );
        let (mut textures, mut fonts, mut world) =
            (Textures::default(), Fonts::default(), World::new());
        let spawned = load(&scene, &mut textures, &mut fonts, &mut world).expect("load");
        assert_eq!(spawned.len(), 3, "an empty entity is still spawned");
        let quad = world.entity(spawned[0]).expect("quad entity");
        assert!(
            quad.get::<&Transform2D>().is_none(),
            "no transform: not drawn, as in code"
        );
        let sprite_entity = world.entity(spawned[1]).expect("sprite entity");
        assert_eq!(
            *sprite_entity.get::<&Transform2D>().expect("transform"),
            Transform2D::IDENTITY
        );
        let sprite = *sprite_entity.get::<&Sprite>().expect("sprite");
        assert_eq!((sprite.tint, sprite.region), (Color::WHITE, None));
        assert_eq!(
            textures.get(sprite.texture).expect("texture").filter,
            TextureFilter::Nearest
        );
        assert!(!sprite_entity.has::<Layer>() && !sprite_entity.has::<Hidden>());
    }

    #[test]
    fn bad_files_are_rejected_with_a_reason() {
        let err = from_text("(version: 2, entities: [])").expect_err("future version");
        assert!(
            err.to_string().contains("version 2 is not supported"),
            "{err}"
        );
        let err = from_text("not ron at all").expect_err("garbage");
        assert!(err.to_string().contains("not a PurplePie scene"), "{err}");
        let err = from_text("(entities: [])").expect_err("no version");
        assert!(err.to_string().contains("not a PurplePie scene"), "{err}");
        let err = from_text(
            "(version: 1, entities: [(quad: (size: (1.0, 1.0), colour: (1.0, 1.0, 1.0, 1.0)))])",
        )
        .expect_err("typo");
        assert!(
            err.to_string().contains("colour"),
            "unknown fields are named: {err}"
        );
        assert!(from_text("(version: 1, entities: [], extra: 1)").is_err());
    }

    #[test]
    fn a_missing_asset_fails_before_anything_is_spawned() {
        let text = r#"(version: 1, entities: [
            (quad: (size: (1.0, 1.0), color: (1.0, 1.0, 1.0, 1.0))),
            (sprite: (texture: "textures/no-such.png", size: (1.0, 1.0))),
        ])"#;
        let scene = from_text(text).expect("parse");
        let (mut textures, mut fonts, mut world) =
            (Textures::default(), Fonts::default(), World::new());
        let err = load(&scene, &mut textures, &mut fonts, &mut world).expect_err("missing");
        assert!(
            matches!(&err, crate::Error::Asset { path, .. } if path.ends_with("textures/no-such.png")),
            "{err}"
        );
        assert!(err.source().is_some());
        assert_eq!(world.len(), 0, "nothing spawned");
    }

    #[test]
    fn the_shipped_demo_scene_loads_and_saves_back_unchanged() {
        let path = root().join("scenes/demo.ron");
        // Git may check text files out with CRLF line ends on Windows.
        let text = std::fs::read_to_string(&path)
            .expect("demo.ron")
            .replace("\r\n", "\n");
        let scene = from_text(&text).expect("parse");
        let (mut textures, mut fonts, mut world) =
            (Textures::default(), Fonts::default(), World::new());
        let spawned = load(&scene, &mut textures, &mut fonts, &mut world).expect("load");
        assert!(spawned.len() >= 10, "{}", spawned.len());
        let again = capture(&world, &textures, &fonts, Some(&root())).expect("capture");
        assert_eq!(
            to_text(&again).expect("write"),
            text,
            "the shipped file is exactly what save_scene writes"
        );
    }
}
