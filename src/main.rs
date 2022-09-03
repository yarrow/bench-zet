#![deny(warnings)]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports), warn(warnings))]
#![deny(unused_must_use)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
use anyhow::{Context, Result};

use std::fs::{self, File};
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};

const SIZES: [u64; 4] = [6, 12, 24, 36];

fn main() -> Result<()> {
    fs::write("compare.sh", comparison_script())?;
    for size in SIZES {
        make_large_file(size)?;
    }
    Ok(())
}
fn effective_size(path: &str) -> u64 {
    if let Ok(data) = fs::metadata(path) {
        data.len()
    } else {
        0
    }
}
const INPUT_PATH: &str = "a.csv";
fn path_for(gigabytes: u64) -> String {
    format!("{gigabytes}G.csv")
}
fn make_large_file(gigabytes: u64) -> Result<()> {
    let output_path = path_for(gigabytes);
    let goal_size = gigabytes * 1024 * 1024 * 1024;

    let start = '!' as u32;
    let fin = '~' as u32;
    let tags: Vec<String> = (start..=fin)
        .map(|x| char::from_u32(x).unwrap().to_string())
        .collect();

    let mut input = BufReader::new(
        File::open(INPUT_PATH).with_context(|| format!("Error opening file `{INPUT_PATH}`"))?,
    );

    if effective_size(&output_path) >= goal_size {
        eprintln!("Warning: File `{output_path}` already exists");
        return Ok(());
    }
    let mut output = BufWriter::new(
        File::create(&output_path)
            .with_context(|| format!("Error opening file `{output_path}`"))?,
    );

    let mut line = String::new();
    let mut output_size = 0u64;
    'out: loop {
        let line_length = input
            .read_line(&mut line)
            .with_context(|| format!("Error reading file `{INPUT_PATH}`"))?
            as u64;
        if line_length == 0u64 {
            break 'out;
        }
        for tag in &tags {
            output.write_all(tag.as_bytes())?;
            output.write_all(line.as_bytes())?;
            output_size += line_length + 1u64;
            if output_size >= goal_size {
                break 'out;
            }
        }
        line.clear();
    }
    output.flush()?;
    Ok(())
}

#[cfg(target_os = "macos")]
const TIME: &str = "/opt/homebrew/bin/gtime";

#[cfg(target_os = "macos")]
const PURGE: &str = "sync; sudo purge";

const ZET: &str = "./zet-0.2.5";
const STATS: &str = "comparison.txt";

fn timed(command: &str, arguments: &str) -> String {
    format!("{PURGE}\n{TIME} -f '%e %F %M %C' {command} {arguments} > /dev/null 2>>{STATS}\n")
}
fn compare(zet_subcommand: &str, unix_command: &str, arguments: &str) -> String {
    let mut commands = timed(&format!("{ZET} {zet_subcommand}"), arguments);
    commands.push_str(&timed(unix_command, arguments));
    commands
}
fn comparison_script() -> String {
    let mut script: Vec<String> = vec![
        format!("cp /dev/null {STATS}\n"),
        compare("union", "uniq", "sorted-a.csv"),
        compare("single", "uniq -u", "sorted-a.csv"),
        compare("multiple", "uniq -d", "sorted-a.csv"),
        compare("intersect", "comm -12", "sorted-a.csv sorted-b.csv"),
        compare("diff", "comm -23", "sorted-a.csv sorted-b.csv"),
        compare("union", "./huniq.sh", "a.csv b.csv c.csv"),
    ];
    for size in SIZES {
        script.push(compare("union", "./huniq.sh", &path_for(size)));
    }
    #[cfg(target_os = "macos")]
    script.push(format!(
        "/usr/bin/time -lp {ZET} union 36G.csv > /dev/null 2>mac-zet-stats.txt\n"
    ));
    script.join("")
}
