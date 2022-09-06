use crate::LoadedFont;
use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver};
use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Deref)]
struct StreamReceiver(Receiver<u32>);
pub struct StreamEvent(pub u32);

pub struct JsRuntimePlugin;

impl Plugin for JsRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(read_stream);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let (tx, rx) = bounded::<u32>(10);
    std::thread::spawn(move || loop {
        // Everything here happens in another thread
        // This is where you could connect to an external data source
        let mut rng = rand::thread_rng();
        let start_time = Instant::now();
        let duration = Duration::from_secs_f32(rng.gen_range(0.0..0.2));
        while start_time.elapsed() < duration {
            // Spinning for 'duration', simulating doing hard work!
        }

        tx.send(rng.gen_range(0..2000)).unwrap();
    });

    commands.insert_resource(StreamReceiver(rx));
    commands.insert_resource(LoadedFont(asset_server.load("fonts/FiraSans-Bold.ttf")));
}

// This system reads from the receiver and sends events to Bevy
fn read_stream(receiver: ResMut<StreamReceiver>, mut events: EventWriter<StreamEvent>) {
    for from_stream in receiver.try_iter() {
        events.send(StreamEvent(from_stream));
    }
}
