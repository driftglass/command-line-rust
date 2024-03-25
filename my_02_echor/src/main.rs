use clap::{Arg, ArgAction, Command};

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
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .get_matches();

    let text: Vec<String> = matches.get_many("text").unwrap().cloned().collect();
    let omit_newline = matches.get_flag("omit_newline");

    // let mut ending = "\n";
    // if omit_newline {
    // ending = "";
    // }

    // 1文で書ける
    // let ending = if omit_newline { "" } else { "\n" };

    // println!("{:#?}", matches);
    // print!("{}{}", text.join(" "), ending);

    // 改行コードを変数に格納しなくてもよい
    print!("{}{}", text.join(" "), if omit_newline { "" } else { "\n" });
}
