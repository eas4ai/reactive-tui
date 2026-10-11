fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut parser = vt100::Parser::new(args[2].parse().unwrap(), args[3].parse().unwrap(), 0);
    parser.process(&std::fs::read(&args[1]).unwrap());
    print!("{}", parser.screen().contents());
}
