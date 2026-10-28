use psudo::{
    fs::init_sandbox,
    lexer::Lexer,
    lsp::start_server,
    ownership::{BorrowChecker, CheckedEnvironment},
    parser::{CheckedInterpret, Interpret, Parser},
    resolver::Resolver,
};
use std::{
    cell::RefCell,
    env::{self},
    fs::{self},
    io::{self, Write},
    path::PathBuf,
    process,
    rc::Rc,
};

enum InterpreterMode {
    Normal(Interpret),
    Checked(CheckedInterpret),
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
#[derive(Default)]
struct Config {
    is_interactive: bool,
    file_path: Option<PathBuf>,
    show_tokens: bool,
    show_ast: bool,
    quiet: bool,
    run_lsp: bool,
    show_syntax: bool,
    check_borrows: bool,
}

fn main() {
    let mut config = Config::default();
    let args: Vec<String> = env::args().collect();

    // Skip program name
    if args.len() <= 1 {
        config.is_interactive = true;
        run_repl(&config);
        return;
    }

    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-V" | "--version" => {
                println!("{} {}", NAME, VERSION);
                return;
            }
            "-i" | "--interactive" => config.is_interactive = true,
            "-f" | "--file" => {
                i += 1;
                if i < args.len() {
                    config.file_path = Some(PathBuf::from(&args[i]));
                } else {
                    eprintln!("Error: --file requires a path argument");
                    process::exit(1);
                }
            }
            "-t" | "--tokens" => config.show_tokens = true,
            "-a" | "--ast" => config.show_ast = true,
            "-q" | "--quiet" => config.quiet = true,
            "-s" | "--syntax" => config.show_syntax = true,
            "-b" | "--check-borrows" => config.check_borrows = true,
            "--lsp" => config.run_lsp = true,
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown argument '{}'", arg);
                print_help();
                process::exit(1);
            }
            _ => {
                // Positional argument = file
                if config.file_path.is_none() {
                    config.file_path = Some(PathBuf::from(arg));
                } else {
                    eprintln!("Error: Multiple input files not supported");
                    process::exit(1);
                }
            }
        }
        i += 1;
    }

    if config.run_lsp {
        if let Err(e) = run_lsp_server() {
            eprintln!("LSP server error: {}", e);
            process::exit(1);
        }
        return;
    }

    if config.show_syntax {
        print_syntax();
        return;
    }

    if config.is_interactive {
        run_repl(&config);
    } else if let Some(path) = &config.file_path {
        run_file(path, &config);
    } else {
        run_repl(&config);
    }
}

fn run_lsp_server() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        start_server().await;
    });
    Ok(())
}

fn print_help() {
    println!(
        r###"{} {} - Lox-inspired interpreter with LSP support
Usage: {} [OPTIONS] [FILE]

Arguments:
  [FILE]    Input file to interpret (.psudo extension)

Config:
  psudo.toml in project root (auto-detected walking up from CWD)
  Controls sandbox root, size limits, external read allow/deny lists

Options:
  -h, --help       Show this help message
  -V, --version    Show version
  -i, --interactive  Start interactive REPL (default if no file)
  -f, --file <PATH>  Input file to interpret
  -t, --tokens       Show tokens after lexing
  -a, --ast          Show AST after parsing
  -q, --quiet        Suppress token/AST output, only show results
  -s, --syntax       Show compressed syntax reference for LLM context
  -b, --check-borrows  Enable borrow checking (move/borrow validation)
      --lsp          Start Language Server Protocol server (stdin/stdout)

Examples:
  {} -f scripts/hello.psudo
  {} -i
  {} -f script.psudo -t -a
  {} -s > note/syntax.md"###,
        NAME, VERSION, NAME, NAME, NAME, NAME, NAME
    );
}
fn print_syntax() {
    let syntax = r#"psudo syntax (compressed):

Literals: num (f32), "str", true/false, nil
Vars: let x = expr; x = expr;
Print: print expr;
Blocks: { stmt; stmt; }
If: if (cond) { ... } else { ... }
While: while (cond) { ... }
For: for (init; cond; inc) { ... }

Ops (prec high->low): - ! (unary,r) * / (l) + - (l) < > <= >= (l) == != (l) = (assign,r)
Str concat: "a" + "b" -> "ab"

Builtins: read_file(path), write_file(path,str), list_dir(path), exists(path), is_file(path), is_dir(path), mkdir(path)
Sandbox: all fs ops confined to ./sandbox/

Comments: // line, /* block */
End: ; required
EOF"#;
    println!("{}", syntax);
}
fn run_file(path: &PathBuf, config: &Config) {
    if !config.quiet {
        println!("📄 Running: {}", path.display());
    }

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path.display(), e);
            process::exit(1);
        }
    };

    run_source(&source, config);
}

fn run_repl(config: &Config) {
    if !config.quiet {
        println!("{} {} - Interactive REPL", NAME, VERSION);
        println!("Type 'exit' or Ctrl+D to quit\n");
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        if !config.quiet {
            print!("> ");
            stdout.flush().ok();
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "exit" || line == "quit" {
            break;
        }

        run_source(line, config);
    }

    if !config.quiet {
        println!("\nBye!");
    }
}

fn run_source(source: &str, config: &Config) {
    // Initialize sandbox
    if let Err(e) = init_sandbox(None) {
        eprintln!("Failed to initialize sandbox: {}", e);
        return;
    }

    let mut lexer = Lexer::new(source.to_owned());
    let tokens = match lexer.scan_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            if !config.quiet {
                println!("Attempting to parse anyway...");
            }
            lexer.get_tokens()
        }
    };

    if config.show_tokens && !config.quiet {
        println!("🔤 Tokens:");
        for token in &tokens {
            println!("  {:?}", token);
        }
        println!();
    }

    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(statements) => statements,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            process::exit(1);
        }
    };
    
    // Run resolver for static analysis
    let resolver = Resolver::new();
    let resolver_result = resolver.resolve(&statements);
    
    // Report resolver errors
    if !resolver_result.errors.is_empty() {
        for error in &resolver_result.errors {
            eprintln!("Resolver error [line {}, col {}]: {}", error.line, error.column, error.message);
        }
        // Continue anyway - interpreter will also catch these
    }
    
    if config.show_ast && !config.quiet {
        println!("🌳 AST:");
        for stmt in &statements {
            println!("  {:?}", stmt);
        }
        println!();
    }
    let mut interpreter = match config.check_borrows {
        true => InterpreterMode::Checked(CheckedInterpret {
            environment: Rc::new(RefCell::new(CheckedEnvironment::new(Rc::new(
                RefCell::new(BorrowChecker::new()),
            )))),
            check_borrows: true,
        }),
        false => InterpreterMode::Normal(Interpret::default()),
    };
    for stmt in statements {
        match &mut interpreter {
            InterpreterMode::Checked(ci) => {
                ci.execute(&stmt);
            }
            InterpreterMode::Normal(ni) => {
                ni.execute(&stmt);
            }
        }
    }
}
