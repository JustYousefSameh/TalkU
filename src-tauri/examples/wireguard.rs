fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("up") => talku_lib::wireguard::up(),
        Some("down") => talku_lib::wireguard::down(),
        _ => {
            eprintln!("usage: wireguard [up|down]");
            std::process::exit(2);
        }
    }
}
