use deno_core::error::AnyError;
use deno_core::op;
use deno_core::Extension;
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;

#[op]
async fn op_log(str: String) -> Result<(), AnyError> {
    println!("[deno] > {}", str);
    Ok(())
}

async fn run_js(file_path: &str, sender: UnboundedSender<u32>) -> Result<(), AnyError> {
    let runjs_extension = Extension::builder()
        .state(move |state| {
            state.put(sender.clone());
            Ok(())
        })
        .ops(vec![op_log::decl()])
        .build();

    let main_module = deno_core::resolve_path(file_path)?;

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs_extension],
        ..Default::default()
    });
    const RUNTIME_JAVASCRIPT_CORE: &str = include_str!("./runtime.js");

    js_runtime
        .execute_script("[runjs:runtime.js]", RUNTIME_JAVASCRIPT_CORE)
        .unwrap();

    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(false).await?;
    result.await?
}

pub fn run_deno_runtime(sender: UnboundedSender<u32>) {
    println!("initializing tokio");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    if let Err(error) = runtime.block_on(run_js("./src/main.js", sender)) {
        eprintln!("error: {}", error);
    }
}
