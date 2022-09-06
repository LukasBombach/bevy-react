use deno_core::anyhow::Error;
use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::RuntimeOptions;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use std::task::Poll;

type JsValue = Box<u32>;

fn main() {
    let my_ext = Extension::builder()
        .ops(vec![op_schedule_task::decl()])
        .event_loop_middleware(|state_rc, cx| {
            let mut state = state_rc.borrow_mut();
            let recv = state.borrow_mut::<mpsc::UnboundedReceiver<JsValue>>();
            let mut ref_loop = false;

            while let Poll::Ready(Some(i)) = recv.poll_next_unpin(cx) {
                println!("got {}", i);
                ref_loop = true; // `call` can callback into runtime and schedule new callbacks :-)
            }
            ref_loop
        })
        .state(move |state| {
            let (tx, rx) = mpsc::unbounded::<JsValue>();
            state.put(tx);
            state.put(rx);

            Ok(())
        })
        .build();

    // Initialize a runtime instance
    let mut js_runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![my_ext],
        ..Default::default()
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let future = async move {
        // Schedule 10 tasks.
        js_runtime
            .execute_script(
                "<usage>",
                r#"for (let i = 1; i <= 10; i++) Deno.core.ops.op_schedule_task(i);"#,
            )
            .unwrap();
        js_runtime.run_event_loop(false).await
    };
    runtime.block_on(future).unwrap();
}

#[op]
fn op_schedule_task(state: &mut OpState, i: u32) -> Result<(), Error> {
    let tx = state.borrow_mut::<mpsc::UnboundedSender<JsValue>>();
    tx.unbounded_send(Box::new(i))
        .expect("unbounded_send failed");
    Ok(())
}
