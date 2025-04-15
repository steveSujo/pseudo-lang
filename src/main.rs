use std::{
    env::{self, Args},
    path::PathBuf,
};

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

    let mut flags: Args = env::args().into_iter();

    while let Some(flag) = flags.next() {
        match flag.as_str() {
            "-i" => program_conf.is_interactive = true,
            "-f" => {
                if let Some(path) = flags.next() {
                    program_conf.file_path = PathBuf::from(path);
                }
            }
            _ => println!("Unknown Argument Passed!"),
        }
    }

    println!(
        "\nprogram config:\n is_interactive: {}\n file_path: {}",
        program_conf.is_interactive,
        program_conf.file_path.display()
    );
}
