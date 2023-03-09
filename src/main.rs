use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to execute (use '-' to read from standard input)
    #[arg(value_name = "FILE", conflicts_with = "command")]
    file: Option<String>,

    /// Command to interpret (conflicts with FILE argument)
    #[arg(short, long, value_name = "TXT", conflicts_with = "file")]
    command: Option<String>,

    /// Set debug option(s) (only works on build with debug features)
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
    let args = Args::parse();
    println!("file: {:?}", args.file);
    println!("command: {:?}", args.command);
    for opt in args.debug_opts {
        println!("-D {:?}", opt);
    }
    println!("arguments {:?}", args.arguments);
}
