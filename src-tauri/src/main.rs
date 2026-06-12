fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        backdrop_lib::run_gui();
    } else {
        backdrop_lib::run_cli(args);
    }
}
