use deno_core::error::AnyError;
use deno_core::op;
use deno_core::Extension;
use std::rc::Rc;

pub async fn run_js_runtime(file_path: &str) -> Result<(), AnyError> {
    const RUNTIME_JAVASCRIPT_CORE: &str = include_str!("./runtime.js");
    let main_module = deno_core::resolve_path(file_path)?;

    let runjs_extension = Extension::builder().ops(vec![op_log::decl()]).build();

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs_extension],
        ..Default::default()
    });

    js_runtime
        .execute_script("runtime.js", RUNTIME_JAVASCRIPT_CORE)
        .unwrap();

    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);

    js_runtime.run_event_loop(false).await?;
    result.await?
}

#[op]
async fn op_log(str: String) -> Result<(), AnyError> {
    println!("[deno] > {}", str);
    Ok(())
}
