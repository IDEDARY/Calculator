use crate::*;

// #=========================#
// #=== PREBUILT COMPONENT ===#

/// When this component is added, a UI system is built
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct MyButton {
    pub text: String,
    pub image: Handle<Image>,
}

/// System that builds the component UI on trigger
fn spawn_observer(
    trigger: Trigger<OnAdd, MyButton>,
    mut commands: Commands, query: Query<&MyButton>,
    asset_server: Res<AssetServer>,
) {
    let Ok(data) = query.get(trigger.entity()) else { return; };
    let entity = trigger.entity();
    commands.entity(entity).insert((
        OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
    )).with_children(|ui| {

        // Spawn the background
        ui.spawn((
            UiLayout::window().full().pack(),
            UiHover::new().forward_speed(20.0).backward_speed(5.0),
            UiColor::new(vec![
                (UiBase::id(), Color::BEVYPUNK_RED.with_alpha(0.4)),
                (UiHover::id(), Color::BEVYPUNK_YELLOW.with_alpha(1.2))
            ]),
            Sprite {
                image: data.image.clone(),
                image_mode: SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::square(32.0), ..default() }),
                ..default()
            },
            PickingBehavior::IGNORE,
        )).with_children(|ui| {

            // Spawn the text
            ui.spawn((
                UiLayout::window().pos(Rl(50.0)).anchor(Anchor::Center).pack(),
                UiColor::new(vec![
                    (UiBase::id(), Color::BEVYPUNK_RED),
                    (UiHover::id(), Color::BEVYPUNK_BLUE.with_alpha(1.2))
                ]),
                UiHover::new().instant(true),
                UiTextSize::from(Rh(50.0)),
                Text2d::new(data.text.to_ascii_uppercase()),
                TextFont {
                    font: asset_server.load("fonts/rajdhani/Rajdhani-Medium.ttf"),
                    font_size: 64.0,
                    ..default()
                },
                PickingBehavior::IGNORE,
                TextPipe(entity),
            ));
        });
    }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

}

/// This component will synchronize the text with [`MyButton`] component
#[derive(Component, Reflect, Deref, DerefMut, Clone, PartialEq, Debug)]
struct TextPipe(pub Entity);
impl TextPipe {
    /// System for syncing text data
    fn system(
        mut commands: Commands,
        src_query: Query<&MyButton, Changed<MyButton>>,
        mut dst_query: Query<(&mut Text2d, &TextPipe)>,
    ) {
        if src_query.is_empty() { return; }
        for (mut text, pipe) in &mut dst_query {
            if let Ok(button) = src_query.get(**pipe) {
                **text = button.text.clone();
                commands.trigger(RecomputeUiLayout);
            }
        }
    }
}


// #========================#
// #=== COMPONENT PLUGIN ===#

/// Plugin adding all our logic
pub struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, TextPipe::system);
        app.add_observer(spawn_observer);
    }
}
