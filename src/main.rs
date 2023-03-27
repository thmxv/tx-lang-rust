use clap::Parser;

// TODO: move to runtime
#[cfg(feature = "debug-features")]
const HAS_DEBUG_FEATURES: bool = true;
#[cfg(not(feature = "debug-features"))]
const HAS_DEBUG_FEATURES: bool = false;

#[derive(Parser, Debug)]
#[command(
    name = option_env!("CARGO_BIN_NAME").unwrap(),
    version,
    about = format!(
        "Tx v{} (rust implementation) (debug features {})\n{}",
        option_env!("CARGO_PKG_VERSION").unwrap(),
        if HAS_DEBUG_FEATURES { "enabled" } else { "disabled" },
        option_env!("CARGO_PKG_DESCRIPTION").unwrap()
    ),
    long_about = None,
)]
struct Args {
    /// File to execute (use '-' to read from standard input)
    #[arg(value_name = "FILE", conflicts_with = "command")]
    file: Option<String>,

    /// Command to interpret (conflicts with FILE argument)
    #[arg(short, long, value_name = "TXT", conflicts_with = "file")]
    command: Option<String>,

    /// Set debug option(s) (requires build with debug features)
    #[arg(short = 'D', value_name = "OPT", value_enum)]
    debug_opts: Vec<DebugOpt>,

    /// Arguments to pass to the interpreted script/command
    #[arg(last = true)]
    arguments: Vec<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum DebugOpt {
    /// Enable all the debug otions
    All,
    /// Print tokens during compilation
    PrintTokens,
    /// Print bytecode after compilation
    PrintBytecode,
    /// Trace bytecode execution
    TraceExecution,
    /// Trace garbage collection
    TraceGC,
}

fn main() {
    print!(r#"
            (o)>    Tx v{}
            //\     MIT License, Copyright (C) 2022-2023 Xavier Thomas
            V_/_    https://github.com/thmxv/tx-lang-rust"#, 
        "TODO");

    let args = Args::parse();
    println!("file: {:?}", args.file);
    println!("command: {:?}", args.command);
    for opt in args.debug_opts {
        println!("-D {:?}", opt);
    }
    println!("arguments {:?}", args.arguments);
}
