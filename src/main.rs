use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process;

#[derive(Clone)]
enum NumberStyle {
    All,             // a: number all lines
    NonEmpty,        // t: number non-empty lines
    None,            // n: no numbering
    Pattern(Regex),  // pBRE: number lines matching regex
}

#[derive(Clone, Copy)]
enum NumberFormat {
    Left,      // ln: left justified
    Right,     // rn: right justified (default)
    RightZero, // rz: right justified, leading zeros
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Header,
    Body,
    Footer,
}

struct Config {
    header_style: NumberStyle,
    body_style: NumberStyle,
    footer_style: NumberStyle,
    number_format: NumberFormat,
    number_width: usize,
    separator: String,
    start_number: i64,
    increment: i64,
    join_blank: usize,
    no_renumber: bool,
    section_delimiter: [char; 2],
    file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            header_style: NumberStyle::None,
            body_style: NumberStyle::NonEmpty,
            footer_style: NumberStyle::None,
            number_format: NumberFormat::Right,
            number_width: 6,
            separator: "\t".to_string(),
            start_number: 1,
            increment: 1,
            join_blank: 1,
            no_renumber: false,
            section_delimiter: ['\\', ':'],
            file: None,
        }
    }
}

fn parse_style(value: &str, option: &str) -> NumberStyle {
    match value {
        "a" => NumberStyle::All,
        "t" => NumberStyle::NonEmpty,
        "n" => NumberStyle::None,
        s if s.starts_with('p') => {
            let pattern = &s[1..];
            match Regex::new(pattern) {
                Ok(re) => NumberStyle::Pattern(re),
                Err(e) => {
                    eprintln!("nl: invalid regex for '{option}': {e}");
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("nl: invalid numbering style: '{value}'");
            process::exit(1);
        }
    }
}

fn require_arg<'a>(args: &'a [String], i: &mut usize, option: &str) -> &'a str {
    *i += 1;
    match args.get(*i) {
        Some(v) => v.as_str(),
        None => {
            eprintln!("nl: option '{option}' requires an argument");
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: nl [OPTION]... [FILE]");
    eprintln!("Write each FILE to standard output, with line numbers added.");
    eprintln!("With no FILE, or when FILE is -, read standard input.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -b STYLE   body line numbering style (default t)");
    eprintln!("  -d CC      section delimiter characters (default \\:)");
    eprintln!("  -f STYLE   footer line numbering style (default n)");
    eprintln!("  -h STYLE   header line numbering style (default n)");
    eprintln!("  -i NUMBER  line number increment (default 1)");
    eprintln!("  -l NUMBER  group of NUMBER empty lines counted as one (default 1)");
    eprintln!("  -n FORMAT  line number format (ln, rn, rz) (default rn)");
    eprintln!("  -p         do not reset line numbers for each section");
    eprintln!("  -s STRING  use STRING as separator after number (default TAB)");
    eprintln!("  -v NUMBER  first line number for each section (default 1)");
    eprintln!("  -w NUMBER  use NUMBER columns for line numbers (default 6)");
    eprintln!("      --help display this help and exit");
    eprintln!();
    eprintln!("STYLE is one of:");
    eprintln!("  a      number all lines");
    eprintln!("  t      number only nonempty lines");
    eprintln!("  n      number no lines");
    eprintln!("  pBRE   number only lines that match the basic regular expression BRE");
    eprintln!();
    eprintln!("Sections are delimited by lines containing only the delimiter");
    eprintln!("characters repeated 1 (footer), 2 (body), or 3 (header) times.");
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut config = Config::default();
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--help" => {
                print_usage();
                process::exit(0);
            }
            "-p" => config.no_renumber = true,
            "-b" => {
                let val = require_arg(&args, &mut i, "-b");
                config.body_style = parse_style(val, "-b");
            }
            "-f" => {
                let val = require_arg(&args, &mut i, "-f");
                config.footer_style = parse_style(val, "-f");
            }
            "-h" => {
                let val = require_arg(&args, &mut i, "-h");
                config.header_style = parse_style(val, "-h");
            }
            "-d" => {
                let val = require_arg(&args, &mut i, "-d");
                let chars: Vec<char> = val.chars().collect();
                match chars.len() {
                    1 => config.section_delimiter = [chars[0], ':'],
                    2 => config.section_delimiter = [chars[0], chars[1]],
                    _ => {
                        eprintln!("nl: invalid section delimiter: '{val}'");
                        process::exit(1);
                    }
                }
            }
            "-n" => {
                let val = require_arg(&args, &mut i, "-n");
                config.number_format = match val {
                    "ln" => NumberFormat::Left,
                    "rn" => NumberFormat::Right,
                    "rz" => NumberFormat::RightZero,
                    _ => {
                        eprintln!("nl: invalid line number format: '{val}'");
                        process::exit(1);
                    }
                };
            }
            "-s" => {
                let val = require_arg(&args, &mut i, "-s");
                config.separator = val.to_string();
            }
            "-w" => {
                let val = require_arg(&args, &mut i, "-w");
                config.number_width = match val.parse() {
                    Ok(w) if w > 0 => w,
                    _ => {
                        eprintln!("nl: invalid line number field width: '{val}'");
                        process::exit(1);
                    }
                };
            }
            "-v" => {
                let val = require_arg(&args, &mut i, "-v");
                config.start_number = match val.parse() {
                    Ok(v) => v,
                    _ => {
                        eprintln!("nl: invalid starting line number: '{val}'");
                        process::exit(1);
                    }
                };
            }
            "-i" => {
                let val = require_arg(&args, &mut i, "-i");
                config.increment = match val.parse() {
                    Ok(inc) => inc,
                    _ => {
                        eprintln!("nl: invalid line number increment: '{val}'");
                        process::exit(1);
                    }
                };
            }
            "-l" => {
                let val = require_arg(&args, &mut i, "-l");
                config.join_blank = match val.parse() {
                    Ok(l) if l > 0 => l,
                    _ => {
                        eprintln!("nl: invalid line number of blank lines: '{val}'");
                        process::exit(1);
                    }
                };
            }
            // Support combined forms like -ba, -bt, -bn, -nln, -nrn, -nrz
            s if s.starts_with("-b") && s.len() > 2 => {
                config.body_style = parse_style(&s[2..], "-b");
            }
            s if s.starts_with("-f") && s.len() > 2 && !s.starts_with("-fo") => {
                config.footer_style = parse_style(&s[2..], "-f");
            }
            s if s.starts_with("-h") && s.len() > 2 && !s.starts_with("-he") => {
                config.header_style = parse_style(&s[2..], "-h");
            }
            s if s.starts_with("-n") && s.len() > 2 => {
                config.number_format = match &s[2..] {
                    "ln" => NumberFormat::Left,
                    "rn" => NumberFormat::Right,
                    "rz" => NumberFormat::RightZero,
                    v => {
                        eprintln!("nl: invalid line number format: '{v}'");
                        process::exit(1);
                    }
                };
            }
            s if !s.starts_with('-') || s == "-" => {
                config.file = if s == "-" { None } else { Some(s.to_string()) };
            }
            _ => {
                eprintln!("nl: invalid option '{arg}'");
                eprintln!("Try 'nl --help' for more information.");
                process::exit(1);
            }
        }
        i += 1;
    }

    config
}

fn format_number(num: i64, width: usize, format: NumberFormat) -> String {
    match format {
        NumberFormat::Left => format!("{:<width$}", num),
        NumberFormat::Right => format!("{:>width$}", num),
        NumberFormat::RightZero => format!("{:>0width$}", num),
    }
}

fn should_number(line: &str, style: &NumberStyle) -> bool {
    match style {
        NumberStyle::All => true,
        NumberStyle::NonEmpty => !line.is_empty(),
        NumberStyle::None => false,
        NumberStyle::Pattern(re) => re.is_match(line),
    }
}

/// Build the section delimiter strings from the two-character delimiter.
/// Returns (header_delim, body_delim, footer_delim).
fn section_delimiters(delim: [char; 2]) -> (String, String, String) {
    let pair: String = delim.iter().collect();
    let header = format!("{pair}{pair}{pair}");
    let body = format!("{pair}{pair}");
    let footer = pair;
    (header, body, footer)
}

fn number_lines(reader: impl Read, config: &Config) -> io::Result<()> {
    let buf = BufReader::new(reader);
    let mut line_number = config.start_number;
    let mut out = io::BufWriter::new(io::stdout().lock());

    let mut current_section = Section::Body;
    let mut blank_count: usize = 0;

    let (header_delim, body_delim, footer_delim) = section_delimiters(config.section_delimiter);

    for line in buf.lines() {
        let line = line?;

        // Check for section delimiter (must check longest first)
        if line == header_delim {
            current_section = Section::Header;
            if !config.no_renumber {
                line_number = config.start_number;
            }
            blank_count = 0;
            writeln!(out)?;
            continue;
        }
        if line == body_delim {
            current_section = Section::Body;
            if !config.no_renumber {
                line_number = config.start_number;
            }
            blank_count = 0;
            writeln!(out)?;
            continue;
        }
        if line == footer_delim {
            current_section = Section::Footer;
            if !config.no_renumber {
                line_number = config.start_number;
            }
            blank_count = 0;
            writeln!(out)?;
            continue;
        }

        let style = match current_section {
            Section::Header => &config.header_style,
            Section::Body => &config.body_style,
            Section::Footer => &config.footer_style,
        };

        // Handle join_blank (-l): group consecutive blank lines
        let do_number = if line.is_empty() {
            blank_count += 1;
            if matches!(style, NumberStyle::All) && blank_count >= config.join_blank {
                blank_count = 0;
                true
            } else {
                false
            }
        } else {
            blank_count = 0;
            should_number(&line, style)
        };

        if do_number {
            let num = format_number(line_number, config.number_width, config.number_format);
            writeln!(out, "{}{}{}", num, config.separator, line)?;
            line_number += config.increment;
        } else {
            // Print empty prefix to align with numbered lines
            writeln!(out, "{}{}", " ".repeat(config.number_width), line)?;
        }
    }

    Ok(())
}

fn main() {
    let config = parse_args();

    let result = match &config.file {
        Some(path) => match File::open(path) {
            Ok(file) => number_lines(file, &config),
            Err(e) => {
                eprintln!("nl: {path}: {e}");
                process::exit(1);
            }
        },
        None => number_lines(io::stdin(), &config),
    };

    if let Err(e) = result {
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("nl: {e}");
            process::exit(1);
        }
    }
}
