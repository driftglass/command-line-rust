use crate::TakeValue::*;
use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgAction, Command};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

#[derive(Debug)]
struct Args {
    files: Vec<String>,
    lines: String,
    bytes: Option<String>,
    quiet: bool,
}

static NUM_RE: OnceCell<Regex> = OnceCell::new();

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

// --------------------------------------------------
fn main() {
    if let Err(e) = run(get_args()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

// --------------------------------------------------
fn get_args() -> Args {
    let matches = Command::new("tailr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rust version of `tail`")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .default_value("10"),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Number of bytes"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress headers"),
        )
        .get_matches();

    Args {
        files: matches.get_many("files").unwrap().cloned().collect(),
        lines: matches.get_one("lines").cloned().unwrap(),
        bytes: matches.get_one("bytes").cloned(),
        quiet: matches.get_flag("quiet"),
    }
}

// --------------------------------------------------
fn run(args: Args) -> Result<()> {
    let lines = parse_num(args.lines)
        .map_err(|e| anyhow!("illegal line count -- {e}"))?;

    let bytes = args
        .bytes
        .map(parse_num)
        .transpose()
        .map_err(|e| anyhow!("illegal byte count -- {e}"))?;

    let num_files = args.files.len();
    for (file_num, filename) in args.files.iter().enumerate() {
        match File::open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => {
                if !args.quiet && num_files > 1 {
                    println!(
                        "{}==> {filename} <==",
                        if file_num > 0 { "\n" } else { "" },
                    );
                }

                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                let file = BufReader::new(file);
                if let Some(num_bytes) = &bytes {
                    print_bytes(file, num_bytes, total_bytes)?;
                } else {
                    print_lines(file, &lines, total_lines)?;
                }
            }
        }
    }

    Ok(())
}

// --------------------------------------------------
fn parse_num(val: String) -> Result<TakeValue> {
    let num_re =
        NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap());

    match num_re.captures(&val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("-", |m| m.as_str());
            let num = format!("{sign}{}", caps.get(2).unwrap().as_str());
            if let Ok(val) = num.parse() {
                if sign == "+" && val == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(val))
                }
            } else {
                bail!(val)
            }
        }
        _ => bail!(val),
    }
}

// --------------------------------------------------
// We have to specify the type and assign to a variable here because
// &['+', '-'] has the type &[char; 2], and we want to coerce it to
// a slice, not a reference to an array.
//
// One day in the future we will be able to say
// val.starts_with(['+', '-'].as_slice())
// but array_methods are currently an unstable nightly feature.
//fn parse_num(val: &str) -> Result<TakeValue> {
//    let signs: &[char] = &['+', '-'];
//    let res = val
//        .starts_with(signs)
//        .then(|| val.parse())
//        .unwrap_or_else(|| val.parse().map(i64::wrapping_neg));

//    match res {
//        Ok(num) => {
//            if num == 0 && val.starts_with('+') {
//                Ok(PlusZero)
//            } else {
//                Ok(TakeNum(num))
//            }
//        }
//        _ => Err(From::from(val)),
//    }
//}

// --------------------------------------------------
fn count_lines_bytes(filename: &str) -> Result<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut num_bytes = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }
    Ok((num_lines, num_bytes))
}

// --------------------------------------------------
fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> Result<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }

    Ok(())
}

// --------------------------------------------------
fn print_lines(
    mut file: impl BufRead,
    num_lines: &TakeValue,
    total_lines: i64,
) -> Result<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf));
            }
            line_num += 1;
            buf.clear();
        }
    }

    Ok(())
}

// --------------------------------------------------
fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total > 0 {
                Some(0)
            } else {
                None
            }
        }
        TakeNum(num) => {
            if num == &0 || total == 0 || num > &total {
                None
            } else {
                let start = if num < &0 { total + num } else { num - 1 };
                Some(if start < 0 { 0 } else { start as u64 })
            }
        }
    }
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{
        count_lines_bytes, get_start_index, parse_num, TakeValue::*,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 1);
        assert_eq!(bytes, 24);

        let res = count_lines_bytes("tests/inputs/twelve.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 12);
        assert_eq!(bytes, 63);
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num((i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
