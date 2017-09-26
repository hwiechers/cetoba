extern crate chess_pgn_parser;
#[macro_use]
extern crate clap;
extern crate special;

mod plot;
mod stats;

use plot::{print_dirichlet_plot_svg, print_scatter_plot_svg};
use stats::fit_polya;

use chess_pgn_parser::GameTermination;
use clap::{App, Arg};

use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir, File};
use std::io::{Read, Write};
use std::iter::Iterator;
use std::path::Path;
use std::process::exit;

type Result<T> = std::result::Result<T, Box<Error>>;

fn main() {
    let matches = App::new("Chess Engine Test Opening Book Analyser")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Analyzes opening books based on engine self-play")
        .arg(
            Arg::with_name("INPUT")
                .help("A PGN file containing the engine self-play results")
                .required(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("The path to output the analysis")
                .required(true),
        )
        .get_matches();

    let pgn_file = matches.value_of("INPUT").expect("Required by clap");
    let output_dir = matches.value_of("OUTPUT").expect("Required by clap");

    run(pgn_file, output_dir).unwrap_or_else(|err| {
        eprintln!("Something when wrong! Error: {}", err);
        exit(1);
    });
}

#[derive(Eq, Hash, PartialEq)]
pub struct OpeningResult {
    white_win_count: u32,
    draw_count: u32,
    black_win_count: u32,
}

impl OpeningResult {
    fn total_games(&self) -> u32 {
        self.white_win_count + self.draw_count + self.black_win_count
    }
    fn white_win_proportion(&self) -> f64 {
        f64::from(self.white_win_count) / f64::from(self.total_games())
    }
    fn draw_proportion(&self) -> f64 {
        f64::from(self.draw_count) / f64::from(self.total_games())
    }
    fn black_win_proportion(&self) -> f64 {
        f64::from(self.black_win_count) / f64::from(self.total_games())
    }
}

fn run(input: &str, output: &str) -> Result<()> {
    let mut file = File::open(input)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let games = chess_pgn_parser::read_games(&contents).map_err(|_| "A parse error occurred")?;

    let output_path = Path::new(output);
    create_dir(output_path)?;

    let total_games = games.len() as u32;
    println!("Total games: {}", total_games);

    let mut opening_stats = HashMap::new();
    for game in games {
        let fen_tags: Vec<&String> = game.tags
            .iter()
            .filter(|&&(ref name, _)| name == "FEN")
            .map(|&(_, ref value)| value)
            .collect();

        let len = fen_tags.len();
        let fen = match len {
            0 => return Err(From::from("FEN tag not found")),
            1 => fen_tags[0].clone(),
            _ => return Err(From::from("Too many FEN tags found")),
        };

        let entry = opening_stats.entry(fen).or_insert(OpeningResult {
            white_win_count: 0,
            draw_count: 0,
            black_win_count: 0,
        });

        match game.termination {
            GameTermination::WhiteWins => {
                entry.white_win_count += 1;
            }
            GameTermination::DrawnGame => {
                entry.draw_count += 1;
            }
            GameTermination::BlackWins => {
                entry.black_win_count += 1;
            }
            _ => {
                return Err(From::from("Bad game termination found"));
            }
        }
    }

    println!("Total openings: {}", opening_stats.len());
    print_opening_stats(
        File::create(output_path.join("opening_stats.csv"))?,
        &opening_stats,
    )?;

    let mut wdb_counts = HashMap::new();
    for result in opening_stats.values() {
        let entry = wdb_counts.entry(result).or_insert(0);
        *entry += 1;
    }

    print_wdb_counts(
        File::create(output_path.join("wdb_counts.csv"))?,
        &wdb_counts,
    )?;
    print_scatter_plot_svg(
        File::create(output_path.join("scatter_plot.svg"))?,
        &wdb_counts,
    )?;

    let samples: Vec<[u32; 3]> = opening_stats
        .values()
        .map(|results| {
            [
                results.white_win_count,
                results.draw_count,
                results.black_win_count,
            ]
        })
        .collect();

    let alpha = fit_polya(&samples);
    println!(
        "Fitted Dirichlet Alpha: ({:.3}, {:.3}, {:.3})",
        alpha[0], alpha[1], alpha[2]
    );

    print_dirichlet_plot_svg(
        File::create(output_path.join("dirichlet_contour_plot.svg"))?,
        &alpha,
    )?;

    Ok(())
}

fn print_opening_stats<T: Write>(
    mut file: T,
    opening_stats: &HashMap<String, OpeningResult>,
) -> Result<()> {
    writeln!(&mut file, "FEN,total,white_win,draw,black_win")?;
    for (fen, result) in opening_stats {
        writeln!(
            &mut file,
            "{},{},{},{},{}",
            fen.split(' ')
                .next()
                .expect("split always has one at least one value"),
            result.total_games(),
            result.white_win_proportion(),
            result.draw_proportion(),
            result.black_win_proportion(),
        )?;
    }
    Ok(())
}

fn print_wdb_counts<T: Write>(
    mut file: T,
    wdb_counts: &HashMap<&OpeningResult, u32>,
) -> Result<()> {
    writeln!(&mut file, "WDB,Count")?;
    for (result, count) in wdb_counts {
        writeln!(
            &mut file,
            "{}-{}-{},{}",
            result.white_win_proportion(),
            result.draw_proportion(),
            result.black_win_proportion(),
            count
        )?;
    }
    Ok(())
}
