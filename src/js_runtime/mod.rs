use bevy::prelude::*;
use deno_core::anyhow::Error;
use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::RuntimeOptions;
use futures::channel::mpsc;
use futures::channel::mpsc::UnboundedReceiver;
use std::ops::Deref;
use std::ops::DerefMut;

// BEVY https://github.com/bevyengine/bevy/blob/main/examples/async_tasks/external_source_external_thread.rs
// DENO https://github.com/denoland/deno/blob/main/core/examples/schedule_task.rs

type JsValue = Box<u32>;

struct StreamReceiver(UnboundedReceiver<JsValue>);

impl Deref for StreamReceiver {
    type Target = UnboundedReceiver<JsValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StreamReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub struct StreamEvent(pub u32);

pub struct JsRuntimePlugin;

impl Plugin for JsRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(read_stream);

        fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
            commands.spawn_bundle(Camera2dBundle::default());
            // commands.insert_resource(StreamReceiver(rx));

            let my_ext = Extension::builder()
                .ops(vec![op_stream_event::decl()])
                .state(move |state| {
                    let (tx, rx) = mpsc::unbounded::<JsValue>();
                    state.put(tx);
                    state.put(rx);
                    Ok(())
                })
                .build();

            let mut v8_runtime = JsRuntime::new(RuntimeOptions {
                extensions: vec![my_ext],
                ..Default::default()
            });

            let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let future = async move {
                v8_runtime
                    .execute_script(
                        "<usage>",
                        r#"for (let i = 1; i <= 10; i++) Deno.core.ops.op_stream_event(i);"#,
                    )
                    .unwrap();
                v8_runtime.run_event_loop(false).await
            };
            tokio_runtime.block_on(future).unwrap();

            #[op]
            fn op_stream_event(state: &mut OpState, i: u32) -> Result<(), Error> {
                let state_tx = state.borrow_mut::<mpsc::UnboundedSender<JsValue>>();
                state_tx
                    .unbounded_send(Box::new(i))
                    .expect("unbounded_send failed");
                Ok(())
            }
        }

        fn read_stream(mut receiver: ResMut<StreamReceiver>, mut events: EventWriter<StreamEvent>) {
            loop {
                if let Ok(opt_i) = receiver.try_next() {
                    if let Some(i) = opt_i {
                        events.send(StreamEvent(*i));
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
