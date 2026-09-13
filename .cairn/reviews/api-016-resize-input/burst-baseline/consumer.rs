use std::io::{self, Write};
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode};
fn main() {
 crossterm::terminal::enable_raw_mode().unwrap();
 event::poll(Duration::ZERO).unwrap();
 println!("READY"); io::stdout().flush().unwrap();
 std::thread::sleep(Duration::from_millis(300));
 let end=Instant::now()+Duration::from_secs(2);
 let mut count=0;
 while Instant::now()<end && count<2048 {
  if event::poll(Duration::from_millis(100)).unwrap() {
   if let Event::Key(k)=event::read().unwrap() { if k.code==KeyCode::Char('a') {count+=1;} }
  }
 }
 crossterm::terminal::disable_raw_mode().unwrap();
 println!("COUNT {count}");
 std::process::exit(if count==2048 {0} else {1});
}
