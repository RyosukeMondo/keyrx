use rhai::{Engine, EvalAltResult};

#[no_mangle]
pub extern "C" fn keyrx_init() {
    println!("KeyRx Core Initializing...");
    
    let mut engine = Engine::new();
    
    // Register custom types and functions
    engine.register_fn("print_debug", |x: i64| {
        println!("Debug from script: {}", x);
    });

    let script = r#"
        print("Rhai Engine Started!");
        print_debug(42);
    "#;

    if let Err(e) = engine.run(script) {
        eprintln!("Script Error: {}", e);
    }
}
