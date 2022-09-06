use crate::LoadedFont;
use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver};
use deno_core::error::AnyError;
use rand::Rng;
use std::rc::Rc;
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
    std::thread::spawn(move || {
        // let mut rng = rand::thread_rng();
        // let start_time = Instant::now();
        // let duration = Duration::from_secs_f32(rng.gen_range(0.0..0.2));
        // while start_time.elapsed() < duration {
        // }
        // tx.send(rng.gen_range(0..2000)).unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        if let Err(error) = runtime.block_on(run_js("./src/main.js")) {
            eprintln!("error: {}", error);
        }
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

async fn run_js(file_path: &str) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path)?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(false).await?;
    result.await?
}
