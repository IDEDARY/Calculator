use crate::*;


// #=========================#
// #=== EXPOSED COMPONENT ===#

/// When this component is added, a UI system is built
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct Button {
    pub text: String,
    pub image: Handle<Image>,
    pub hover_enlarge: bool,
}


// #===============================#
// #=== SANDBOXED USER INTEFACE ===#

/// Marker struct for the sandboxed UI
#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ButtonUi;

/// System that builds the component UI
fn build_component (mut commands: Commands, query: Query<(Entity, &Button), Added<Button>>, assets: Res<AssetServer>) {
    for (entity, button_source) in &query {

        // Spawn button text
        let text = commands.spawn((
            // Link this widget
            UiLink::<ButtonUi>::path("Control/Image/Text"),

            // Position text
            UiLayout::window().pos(Rl((50., 50.))).anchor(Anchor::Center).pack::<Base>(),

            // Add text
            UiText2dBundle {
                text: Text::from_section(&button_source.text,
                    TextStyle {
                        font: assets.load(AssetPath::FONT_MEDIUM),
                        font_size: 60.0,
                        ..default()
                    }),
                ..default()
            },

            // Text size
            UiTextSize::new().size(Rh(60.0)),

            // Make it non-obsructable for hit checking (mouse detection)
            Pickable::IGNORE,

            // This is required to control our hover animation
            UiAnimator::<Hover>::new().receiver(true),

            // This will set the color to red
            UiColor::<Base>::new(Color::BEVYPUNK_RED),

            // This will set hover color to yellow
            UiColor::<Hover>::new(Color::BEVYPUNK_YELLOW),
        )).id();

        // This will create a private sandboxed UiTree within the entity just for the button
        commands.entity(entity).insert(
            UiTreeBundle::<ButtonUi>::from(UiTree::new("Button")),
        ).with_children(|ui| {

            // Spawn button image
            let image = ui.spawn((
                // Link this widget
                UiLink::<ButtonUi>::path("Control/Image"),

                // Add layout
                UiLayout::window_full().pack::<Base>(),

                // Give it a background image
                UiImage2dBundle::from(button_source.image.clone()),

                // Make the background scalable
                ImageScaleMode::Sliced(TextureSlicer { border: BorderRect::square(32.0), ..default() }),

                // Make it non-obsructable for hit checking (mouse detection)
                Pickable::IGNORE,

                // This is required to control our hover animation
                UiAnimator::<Hover>::new().receiver(true),

                // This will set the base color to red
                UiColor::<Base>::new(Color::BEVYPUNK_RED),

                // This will set hover color to yellow
                UiColor::<Hover>::new(Color::BEVYPUNK_YELLOW),

                // Set hover layout
                UiLayout::boundary()
                    .pos1(Rl(if button_source.hover_enlarge { -5.0 } else { 0.0 }))
                    .pos2(Rl(if button_source.hover_enlarge { 105.0 } else { 100.0 }))
                    .pack::<Hover>(),

                UiLayoutController::default(),

            )).id();

            // Spawn button hover-zone
            ui.spawn((
                // Link this widget
                UiLink::<ButtonUi>::path("Control"),

                // Add layout
                UiLayout::window_full().pack::<Base>(),

                // Make this spacial & clickable entity
                UiZoneBundle::default(),

                // This is required to control our hover animation
                UiAnimator::<Hover>::new().forward_speed(6.0).backward_speed(3.0),

                // This will pipe this hover data to the specified entities
                UiAnimatorPipe::<Hover>::new(vec![text, image]),

                // This will change cursor icon on mouse hover
                OnHoverSetCursor::new(CursorIcon::Pointer),

                // Play sound on hover event
                OnHoverPlaySound::new(assets.load(AssetPath::SFX_UI)),

                // If we click on this hover zone, it will emmit UiClick event from parent entity
                UiClickEmitter::new(entity),
            ));
        }).insert(
            TextPipe { entity: vec![text]}
        ).push_children(&[text]);
    }
}


#[derive(Component)]
struct TextPipe {
    entity: Vec<Entity>
}
fn pipe_text(
    query: Query<(&Button, &TextPipe), Changed<Button>>,
    mut desti: Query<&mut Text>,
) {
    for (button, pipe) in &query {
        for e in &pipe.entity {
            if let Ok(mut text) = desti.get_mut(*e) {
                text.sections[0].value = button.text.clone();
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
        app
            // Add Lunex plugins for our sandboxed UI
            .add_plugins(UiGenericPlugin::<ButtonUi>::new())
            //.add_plugins(UiDebugPlugin::<ButtonUi>::new())

            .add_systems(Update, pipe_text)

            // Add general systems
            .add_systems(Update, build_component.before(UiSystems::Compute));
    }
}
