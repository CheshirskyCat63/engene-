use game_framework::EngineRuntimeAssembly;

fn main() {
    println!("Engene Run - Headless Engine Runtime");
    
    // Create and run headless engine
    let mut runtime = EngineRuntimeAssembly::kernel_headless();
    
    // Run a few ticks to demonstrate functionality
    for tick in 0..10 {
        println!("Tick {}", tick);
        runtime.tick(1.0 / 60.0);
    }
    
    println!("Engine runtime completed successfully");
}
