mod js_runtime;

use bevy::prelude::*;
use js_runtime::JsRuntimePlugin;
use js_runtime::StreamEvent;
use rand::Rng;

fn main() {
    App::new()
        .add_event::<StreamEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(JsRuntimePlugin)
        .add_system(spawn_text)
        .add_system(move_text)
        .run();
}

#[derive(Deref)]
struct LoadedFont(Handle<Font>);

fn spawn_text(
    mut commands: Commands,
    mut reader: EventReader<StreamEvent>,
    loaded_font: Res<LoadedFont>,
) {
    let text_style = TextStyle {
        font: loaded_font.clone(),
        font_size: 20.0,
        color: Color::WHITE,
    };

    for (per_frame, event) in reader.iter().enumerate() {
        commands.spawn_bundle(Text2dBundle {
            text: Text::from_section(event.0.to_string(), text_style.clone())
                .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_xyz(
                per_frame as f32 * 100.0 + rand::thread_rng().gen_range(-40.0..40.0),
                300.0,
                0.0,
            ),
            ..default()
        });
    }
}

fn move_text(
    mut commands: Commands,
    mut texts: Query<(Entity, &mut Transform), With<Text>>,
    time: Res<Time>,
) {
    for (entity, mut position) in &mut texts {
        position.translation -= Vec3::new(0.0, 100.0 * time.delta_seconds(), 0.0);
        if position.translation.y < -300.0 {
            commands.entity(entity).despawn();
        }
    }
}
