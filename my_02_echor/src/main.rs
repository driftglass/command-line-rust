use clap::{Arg, Command};

fn main() {
    // println!(std::env::args()); // error
    // println!("{:?}", std::env::args());
    let matches = Command::new("echor")
        .version("0.1.0")
        .author("driftglass <mail@gmail.com>")
        .about("Rust echo")
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .help("Input text")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .help("Do not print newline")
                .required(false),
        )
        .get_matches();

    println!("{:#?}", matches);
}
