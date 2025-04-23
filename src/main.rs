use std::{
    env::{self, Args},
    fs::{self},
    io,
    path::PathBuf,
};

use psudo::{lexer::Lexer, tokens::Token};

#[derive(Default)]
struct Config {
    is_interactive: bool,
    file_path: PathBuf,
}

// impl Default for Config {
//     fn default() -> Self {
//         Config {
//             is_interactive: false,
//             file_path: PathBuf::def,
//         }
//     }
// }
fn main() {
    let mut program_conf: Config = Config::default();

    //TODO: refactor CLI
    let mut flags: Args = env::args().into_iter();

    //skip the default exe path arg
    flags.next();

    if flags.len() >= 1 {
        while let Some(flag) = flags.next() {
            match flag.as_str() {
                "-i" | "--interactive" => program_conf.is_interactive = true,
                "-f" | "--file" => {
                    if let Some(path) = flags.next() {
                        program_conf.file_path = PathBuf::from(path);
                    }
                }
                "-h" | "--help" => {
                    print_help();
                    break;
                }
                _ => println!("Unknown Argument Passed!"),
            }
        }
    } else {
        //start interactive promtp
        program_conf.is_interactive = true;
    }

    match program_conf {
        Config {
            is_interactive: true,
            ..
        } => run_prompt(),
        Config {
            is_interactive: false,
            file_path: path,
        } => run_file(path),
    };
}
fn print_help() {
    println!(
        "\nUsage: pseudo [Option] \
             \n\nOptions:\
              \n\t-h,--help\t\tDisplay this help\
              \n\t-i,--interactive\tStart interactive Prompt (default if no file is passed)\
              \n\t-f,--file\t\tInput file to interpret\
"
    )
}

fn run_file(path: PathBuf) {
    println!("\nfrom run file {}\n", path.display());

    let source = fs::read_to_string(path).expect("Could not open file");

    run(source);
}

fn run_prompt() {
    loop {
        let mut line = String::new();
        println!(">>");
        io::stdin().read_line(&mut line).expect("fail to read line");

        if line.is_empty() || line == "\n" {
            break;
        };

        run(line)
    }
}

fn run(source: String) {
    let mut lexer = Lexer::new(source.trim().to_owned());
    let token_list = lexer.scan_tokens();

    for token in token_list {
        println!("{}", token);
    }
}
