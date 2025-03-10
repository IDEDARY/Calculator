use std::ops::Add;

pub(crate) use calc::*;
pub(crate) use bevy::core_pipeline::bloom::Bloom;
pub(crate) use bevy::prelude::*;
pub(crate) use bevy_lunex::*;
pub(crate) use bevy_embedded_assets::*;

mod button;
use button::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(EmbeddedAssetPlugin { mode: PluginMode::ReplaceDefault})
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Calculator".into(),
                        resolution: bevy::window::WindowResolution::new(360.0, 520.0),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
            UiLunexPlugins,
        ))

        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_ui)
        .add_plugins(ButtonPlugin)

        .add_systems(Update, vfx_bloom_flicker)
        .run()
}


// #=====================#
// #=== GENERIC SETUP ===#


/// This system spawns & setups the basic camera with cursor
fn spawn_camera(mut commands: Commands, asset_server: Res<AssetServer>, mut atlas_layout: ResMut<Assets<TextureAtlasLayout>>) {
    // Spawn the camera
    commands.spawn((
        Camera2d, Camera { hdr: true, clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)), ..default() }, Bloom::OLD_SCHOOL, UiSourceCamera::<0>, Transform::from_translation(Vec3::Z * 1000.0),
    )).with_children(|cam| {

        // Spawn cursor
        cam.spawn ((
            SoftwareCursor::new()
                .set_index(bevy::window::SystemCursorIcon::Default, 0, (14.0, 14.0))
                .set_index(bevy::window::SystemCursorIcon::Pointer, 1, (10.0, 12.0))
                .set_index(bevy::window::SystemCursorIcon::Grab, 2, (40.0, 40.0)),

            // Change the scale
            Transform::from_scale(Vec3::new(0.45, 0.45, 1.0)),
            
            // Change the sprite
            Sprite {
                image: asset_server.load("images/cursor.png"),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.add(TextureAtlasLayout::from_grid(UVec2::splat(80), 3, 1, None, None)),
                    index: 0,
                }),
                color: Color::BEVYPUNK_YELLOW.with_alpha(1.0),
                anchor: Anchor::TopLeft,
                ..default()
            },
        ));

    });
}

/// This system spawns the user interface
fn spawn_ui(mut commands: Commands, assets: Res<AssetServer>){

    // Spawn the master ui tree
    commands.spawn((
        Name::new("Calculator"),
        // Create the UI Root
        UiLayoutRoot::new_2d(),
        // Make the UI synchronized with camera viewport size
        UiFetchFromCamera::<0>,
    )).with_children(|ui| {


        // Spawn the background
        ui.spawn((
            Name::new("Background"),
            UiLayout::solid().size((2968.0, 1656.0)).scaling(Scaling::Fill).pack(),
            Sprite::from_image(assets.load("images/background.png")),
        ));

        // Spawn the container
        ui.spawn((
            Name::new("Container"),
            UiLayout::solid().size((360.0, 520.0)).scaling(Scaling::Fit).pack(),
        )).with_children(|ui| {

            // Spawn clear button
            ui.spawn((
                Name::new("C"),
                UiLayout::window().pos(get_pos(0, 0)).size(get_size(1, 1)).pack(),
                MyButton {
                    text: "C".into(),
                    image: assets.load("images/button_symetric.png"),
                },
                ActionButton,
            )).observe(action_observer);

            // Spawn text field
            ui.spawn((
                Name::new("Text"),
                UiLayout::window().pos(get_pos(1, 0)).size(get_size(3, 1)).pack(),
                MyButton {
                    text: "".into(),
                    image: assets.load("images/button_symetric.png"),
                },
                DisplayField,
            ));

            for (i, text) in ["7", "8", "9", "/", "4", "5", "6", "*", "1", "2", "3", "-", "0", ".", "=", "+"].iter().enumerate() {
                ui.spawn((
                    Name::new(format!("{i}")),
                    UiLayout::window().pos(get_pos(i % XN as usize, i / XN as usize + 1)).size(get_size(1, 1)).pack(),
                    MyButton {
                        text: text.to_string(),
                        image: assets.load(
                            match *text {
                                "7" => "images/button_sliced_top_left.png",
                                "8" => "images/button_symetric.png",
                                "9" => "images/button_symetric.png",
                                "/" => "images/button_sliced_top_right.png",
                                "4" => "images/button_symetric.png",
                                "5" => "images/button_symetric.png",
                                "6" => "images/button_symetric.png",
                                "*" => "images/button_symetric.png",
                                "1" => "images/button_symetric.png",
                                "2" => "images/button_symetric.png",
                                "3" => "images/button_symetric.png",
                                "-" => "images/button_symetric.png",
                                "0" => "images/button_sliced_bottom_left.png",
                                "." => "images/button_symetric.png",
                                "=" => "images/button_symetric.png",
                                "+" => "images/button_sliced_bottom_right.png",
                                _ => "images/button_symetric.png"
                            }
                        ),
                    },
                    ActionButton,
                )).observe(action_observer);
            }
        });
    });
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

fn action_observer(
    trigger: Trigger<Pointer<Click>>,
    actions: Query<&MyButton, With<ActionButton>>,
    mut display: Single<&mut MyButton, (With<DisplayField>, Without<ActionButton>)>,
) {
    if let Ok(button) = actions.get(trigger.entity()) {
        info!("Pressed: {}", button.text);
        match button.text.as_str() {
            "C" => { display.text.clear() },
            "=" => { 
                if let Ok(result) = Context::<f64>::default().evaluate(&display.text) {
                    display.text = format!("{}", result);
                } else {
                    display.text = String::from("Error");
                }
            },
            _ => {
                if display.text.as_str() == "Error" { display.text.clear() }
                display.text += &button.text
            }
        };
    }
}


// #===================#
// #=== BOILERPLATE ===#

/// Custom color palette
pub trait BevypunkColorPalette {
    const BEVYPUNK_RED: Color;
    const BEVYPUNK_RED_DIM: Color;
    const BEVYPUNK_YELLOW: Color;
    const BEVYPUNK_BLUE: Color;
}
impl BevypunkColorPalette for Color {
    const BEVYPUNK_RED: Color = Color::srgba(1.0, 98./255., 81./255., 1.0);
    const BEVYPUNK_RED_DIM: Color = Color::srgba(172./255., 64./255., 63./255., 1.0);
    const BEVYPUNK_YELLOW: Color = Color::linear_rgba(252./255., 226./255., 8./255., 1.0);
    const BEVYPUNK_BLUE: Color = Color::srgba(8./255., 226./255., 252./255., 1.0);
}

/// VFX bloom flickering
fn vfx_bloom_flicker(mut query: Query<&mut Bloom>) {
    for mut bloom in &mut query {
        let mut rng = rand::thread_rng();
        if rand::Rng::gen_range(&mut rng, 0..100) < 20 {
            bloom.intensity += (rand::Rng::gen_range(&mut rng, 0.20..0.30)-bloom.intensity)/6.0;
            bloom.prefilter.threshold += (rand::Rng::gen_range(&mut rng, 0.20..0.30)-bloom.prefilter.threshold)/4.0;
        }
    }
}