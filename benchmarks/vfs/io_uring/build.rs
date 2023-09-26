fn main() {
    if cfg!(target_os = "linux") {
        // Linux-specific build logic goes here
        println!("Building for Linux");
        // Continue with your build logic for Linux
    } else {
        // Print a message and abort the build for other operating systems
        eprintln!("This project only supports Linux.");
        std::process::exit(1);
    }
}