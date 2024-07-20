use std::ops::Add;

pub(crate) use calc::*;
pub(crate) use bevy::core_pipeline::bloom::BloomSettings;
pub(crate) use bevy::{prelude::*, sprite::Anchor};
pub(crate) use bevy_lunex::prelude::*;
pub(crate) use bevy_embedded_assets::*;
pub(crate) use bevy_kira_audio::prelude::*;

mod button;
use button::*;

fn main() {
    App::new()
        .add_plugins(EmbeddedAssetPlugin { mode: PluginMode::ReplaceDefault})
        .add_plugins(DefaultPlugins.set (
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Calculator".into(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    resolution: bevy::window::WindowResolution::new(360.0, 520.0),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }
            ).set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
        )
        .add_plugins((AudioPlugin, UiPlugin))

        .add_systems(Startup, setup)
        .add_systems(Update, add_controller)
        .add_systems(Update, update.run_if(on_event::<UiClickEvent>()))
        .add_plugins(ButtonPlugin)

        .add_systems(Update, vfx_bloom_flicker)
        .run();
}


// #=====================#
// #=== GENERIC SETUP ===#

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut atlas_layout: ResMut<Assets<TextureAtlasLayout>>){
    // Spawn 2D camera
    commands.spawn((
        MainUi,
        BloomSettings::OLD_SCHOOL,
        InheritedVisibility::default(),
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
    )).with_children(|camera| {

        // Spawn cursor
        camera.spawn ((

            // Here we can map different native cursor icons to texture atlas indexes and sprite offsets
            Cursor2d::new()
                .set_index(CursorIcon::Default, 0, (14.0, 14.0))
                .set_index(CursorIcon::Pointer, 1, (10.0, 12.0))
                .set_index(CursorIcon::Grab, 2, (40.0, 40.0)),

            // Here we specify that the cursor should be controlled by gamepad 0
            //GamepadCursor::new(0),

            // This is required for picking to work
            PointerBundle::new(PointerId::Custom(pointer::Uuid::new_v4())),
            
            // Add texture atlas to the cursor
            TextureAtlas {
                layout: atlas_layout.add(TextureAtlasLayout::from_grid(UVec2::splat(80), 3, 1, None, None)),
                index: 0,
            },
            SpriteBundle {
                texture: assets.load(AssetPath::CURSOR),
                transform: Transform { scale: Vec3::new(0.45, 0.45, 1.0), ..default() },
                sprite: Sprite {
                    color: Color::BEVYPUNK_YELLOW.with_alpha(2.0),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },

            // Make the raycaster ignore this entity, we don't want our cursor to block clicking
            Pickable::IGNORE,
        ));
    });


    // #======================#
    // #=== USER INTERFACE ===#

    // Spawn the master ui tree
    commands.spawn((
        MovableByCamera,
        UiTreeBundle::<MainUi>::from(UiTree::new("Calculator")),
    )).with_children(|ui| {

        // Spawn the root div
        let root = UiLink::<MainUi>::path("Root");
        ui.spawn((
            root.clone(),
            UiLayout::window_full().pack::<Base>(),
        ));

        // Spawn the background
        ui.spawn((
            root.add("Background"),
            UiLayout::solid().size((2968.0, 1656.0)).scaling(Scaling::Fill).pack::<Base>(),
            UiImage2dBundle::from(assets.load(AssetPath::BACKGROUND)),
        ));

        // Spawn the container
        let container = root.add("Container");
        ui.spawn((
            container.clone(),
            UiLayout::solid().size((360.0, 520.0)).scaling(Scaling::Fit).pack::<Base>(),
        ));

        // Spawn clear button
        ui.spawn((
            container.add("C"),
            UiLayout::window().pos(get_pos(0, 0)).size(get_size(1, 1)).pack::<Base>(),
            Button {
                text: "C".into(),
                image: assets.load(AssetPath::BUTTON_SYMETRIC),
                hover_enlarge: true,
            },
            ActionButton,
        ));

        // Spawn text field
        ui.spawn((
            container.add("Text"),
            UiLayout::window().pos(get_pos(1, 0)).size(get_size(3, 1)).pack::<Base>(),
            Button {
                text: "".into(),
                image: assets.load(AssetPath::BUTTON_SYMETRIC),
                hover_enlarge: false,
            },
            DisplayField,
        ));

        for (i, tx) in ["7", "8", "9", "/", "4", "5", "6", "*", "1", "2", "3", "-", "0", ".", "=", "+"].iter().enumerate() {
            ui.spawn((
                container.add(format!("{i}")),
                UiLayout::window().pos(get_pos(i % XN as usize, i / XN as usize + 1)).size(get_size(1, 1)).pack::<Base>(),
                Button {
                    text: tx.to_string(),
                    image: assets.load(
                        match tx {
                            &"7" => AssetPath::BUTTON_SLICED_TOP_LEFT,
                            &"8" => AssetPath::BUTTON_SYMETRIC,
                            &"9" => AssetPath::BUTTON_SYMETRIC,
                            &"/" => AssetPath::BUTTON_SLICED_TOP_RIGHT,
                            &"4" => AssetPath::BUTTON_SYMETRIC,
                            &"5" => AssetPath::BUTTON_SYMETRIC,
                            &"6" => AssetPath::BUTTON_SYMETRIC,
                            &"*" => AssetPath::BUTTON_SYMETRIC,
                            &"1" => AssetPath::BUTTON_SYMETRIC,
                            &"2" => AssetPath::BUTTON_SYMETRIC,
                            &"3" => AssetPath::BUTTON_SYMETRIC,
                            &"-" => AssetPath::BUTTON_SYMETRIC,
                            &"0" => AssetPath::BUTTON_SLICED_BOTTOM_LEFT,
                            &"." => AssetPath::BUTTON_SYMETRIC,
                            &"=" => AssetPath::BUTTON_SYMETRIC,
                            &"+" => AssetPath::BUTTON_SLICED_BOTTOM_RIGHT,
                            _ => AssetPath::BUTTON_SYMETRIC
                        }
                    ),
                    hover_enlarge: true,
                },
                ActionButton,
            ));
        }
    });
}

fn add_controller(
    gamepad: Res<Gamepads>,
    mut commands: Commands,
    query: Query<Entity, With<Cursor2d>>,

) {
    if gamepad.contains(Gamepad::new(0)) {
       commands.entity(query.single()).insert(GamepadCursor::new(0));
    } else {
        commands.entity(query.single()).remove::<GamepadCursor>();
    }
}

// #====================#
// #=== GRID COMPUTE ===#

const XN: f32 = 4.0;
const YN: f32 = 5.0;
const GAP: f32 = 5.0;

fn get_pos(x: usize, y: usize) -> Rl<Vec2> {
    let xslice = (100.0 - GAP * (XN + 1.0))/XN;
    let yslice = (100.0 - GAP * (YN + 1.0))/YN;

    Rl(Vec2::new(
        x.add(1) as f32 * GAP + x as f32 * xslice,
        y.add(1) as f32 * GAP + y as f32 * yslice
    ))
}

fn get_size(colspan: usize, rowspan: usize) -> Rl<Vec2> {
    let xslice = (100.0 - GAP * (XN + 1.0))/XN;
    let yslice = (100.0 - GAP * (YN + 1.0))/YN;

    Rl(Vec2::new(
        colspan as f32 * xslice + colspan.saturating_sub(1) as f32 * GAP,
        rowspan as f32 * yslice + rowspan.saturating_sub(1) as f32 * GAP,
    ))
}


// #=====================#
// #=== FUNCTIONALITY ===#

#[derive(Component)]
pub struct DisplayField;

#[derive(Component)]
pub struct ActionButton;


fn update(
    mut events: EventReader<UiClickEvent>,
    mut display: Query<&mut Button, (With<DisplayField>, Without<ActionButton>)>,
    actions: Query<&Button, With<ActionButton>>
) {
    for event in events.read() {
        if let Ok(btn) = actions.get(event.target) {
            let Ok(mut display) = display.get_single_mut() else { return; };

            match btn.text.as_str() {
                "C" => { display.text.clear() },
                "=" => { 
                    if let Ok(result) = Context::<f64>::default().evaluate(&display.text) {
                        display.text = format!("{}", result);
                    } else {
                        display.text = format!("Error");
                    }
                },
                _ => {
                    if display.text.as_str() == "Error" { display.text.clear() }
                    display.text += &btn.text
                }
            };

        }
    }
}

// #===================#
// #=== BOILERPLATE ===#

/// All asset paths as constants
pub struct AssetPath;
impl AssetPath {
    // Music
    pub const SFX_UI: &'static str = "sounds/ui_ping.ogg";

    // Fonts
    pub const FONT_LIGHT: &'static str = "fonts/rajdhani/Rajdhani-Light.ttf";
    pub const FONT_REGULAR: &'static str = "fonts/rajdhani/Rajdhani-Regular.ttf";
    pub const FONT_MEDIUM: &'static str = "fonts/rajdhani/Rajdhani-Medium.ttf";
    pub const FONT_SEMIBOLD: &'static str = "fonts/rajdhani/Rajdhani-SemiBold.ttf";
    pub const FONT_BOLD: &'static str = "fonts/rajdhani/Rajdhani-Bold.ttf";

    // Cursor
    pub const CURSOR: &'static str = "images/cursor.png";

    // Symbols
    pub const BUTTON_SYMETRIC: &'static str = "images/button_symetric.png";
    pub const BUTTON_SYMETRIC_SLICED: &'static str = "images/button_symetric_sliced.png";
    pub const BUTTON_SLICED_BOTTOM_LEFT: &'static str = "images/button_sliced_bottom_left.png";
    pub const BUTTON_SLICED_BOTTOM_RIGHT: &'static str = "images/button_sliced_bottom_right.png";
    pub const BUTTON_SLICED_TOP_LEFT: &'static str = "images/button_sliced_top_left.png";
    pub const BUTTON_SLICED_TOP_RIGHT: &'static str = "images/button_sliced_top_right.png";
    pub const CHEVRON_LEFT: &'static str = "images/chevron_left.png";
    pub const CHEVRON_RIGHT: &'static str = "images/chevron_right.png";
    pub const SWITCH_BASE: &'static str = "images/switch_base.png";
    pub const SWITCH_HEAD: &'static str = "images/switch_head.png";

    // Miscelanious
    pub const BACKGROUND: &'static str = "images/background.png";
    pub const PANEL: &'static str = "images/panel.png";
}

/// Custom color palette
pub trait BevypunkColorPalette {
    const BEVYPUNK_RED: Color;
    const BEVYPUNK_RED_DIM: Color;
    const BEVYPUNK_YELLOW: Color;
    const BEVYPUNK_BLUE: Color;
}
impl BevypunkColorPalette for Color {
    const BEVYPUNK_RED: Color = Color::srgba(255./255., 98./255., 81./255., 1.0);
    const BEVYPUNK_RED_DIM: Color = Color::srgba(172./255., 64./255., 63./255., 1.0);
    const BEVYPUNK_YELLOW: Color = Color::linear_rgba(252./255., 226./255., 8./255., 1.0);
    const BEVYPUNK_BLUE: Color = Color::srgba(8./255., 226./255., 252./255., 1.0);
}

/// VFX bloom flickering
fn vfx_bloom_flicker(mut query: Query<&mut BloomSettings>) {
    for mut bloom in &mut query {
        let mut rng = rand::thread_rng();
        if rand::Rng::gen_range(&mut rng, 0..100) < 20 {
            bloom.intensity += (rand::Rng::gen_range(&mut rng, 0.20..0.30)-bloom.intensity)/6.0;
            bloom.prefilter_settings.threshold += (rand::Rng::gen_range(&mut rng, 0.20..0.30)-bloom.prefilter_settings.threshold)/4.0;
        }
    }
}