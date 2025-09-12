use reactive_tui::backend::{CrosstermBackend, Backend};

fn main() -> reactive_tui::Result<()> {
    println!("Testing terminal initialization...");
    
    // Check if we're in a TTY
    if !atty::is(atty::Stream::Stdout) {
        println!("Not running in a TTY - this might be the issue!");
        return Ok(());
    }
    
    println!("Running in TTY, creating backend...");
    
    // Create backend (this should initialize the terminal)
    let mut backend = CrosstermBackend::new()?;
    println!("Backend created successfully!");
    
    println!("Testing event polling with short timeout...");
    
    // Test polling with a very short timeout
    match backend.poll_event(Some(100)) {
        Ok(Some(event)) => {
            println!("Got event: {:?}", event);
        }
        Ok(None) => {
            println!("No event within 100ms - this is normal");
        }
        Err(e) => {
            println!("Error polling event: {}", e);
            return Err(e);
        }
    }
    
    println!("Event polling test completed successfully!");
    println!("Press any key within 2 seconds to test event reading...");
    
    // Test with a longer timeout to see if we can get an event
    match backend.poll_event(Some(2000)) {
        Ok(Some(event)) => {
            println!("Got event: {:?}", event);
        }
        Ok(None) => {
            println!("No event within 2 seconds");
        }
        Err(e) => {
            println!("Error polling event: {}", e);
            return Err(e);
        }
    }
    
    println!("Terminal test completed!");
    Ok(())
}
