use crate::LoadedFont;
use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
// use crossbeam_channel::unbounded;
use deno_core::error::AnyError;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
// use futures::channel::mpsc;
// use futures::channel::mpsc::UnboundedReceiver;
use std::rc::Rc;

type Num = Box<u32>;

// THREADS
// BEVY https://github.com/bevyengine/bevy/blob/main/examples/async_tasks/external_source_external_thread.rs
// DENO https://github.com/denoland/deno/blob/main/core/examples/schedule_task.rs

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
    let (tx, rx) = bounded::<u32>(10);

    commands.spawn_bundle(Camera2dBundle::default());
    commands.insert_resource(StreamReceiver(rx));
    commands.insert_resource(LoadedFont(asset_server.load("fonts/FiraSans-Bold.ttf")));

    let my_ext = Extension::builder()
        .ops(vec![op_stream_event::decl()])
        .state(move |state| {
            state.put(tx);
            Ok(())
        })
        .build();

    #[op]
    fn op_stream_event(state: &mut OpState, num: u32) -> Result<(), AnyError> {
        let state_tx = state.borrow_mut::<Sender<u32>>();
        state_tx.send(num).expect("unbounded_send failed");
        Ok(())
    }

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            extensions: vec![my_ext],
            ..Default::default()
        });

        if let Err(error) = runtime.block_on(async move {
            let main_module = deno_core::resolve_path("./src/main.js")?;

            js_runtime
                .execute_script("[runjs:runtime.js]", include_str!("./runtime.js"))
                .unwrap();
            let mod_id = js_runtime.load_main_module(&main_module, None).await?;
            let result = js_runtime.mod_evaluate(mod_id);
            js_runtime.run_event_loop(false).await?;
            result.await?
        }) {
            eprintln!("error: {}", error);
        }
    });
}

// This system reads from the receiver and sends events to Bevy
fn read_stream(receiver: ResMut<StreamReceiver>, mut events: EventWriter<StreamEvent>) {
    for from_stream in receiver.try_iter() {
        events.send(StreamEvent(from_stream));
    }
}
