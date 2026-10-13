use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    query: String,
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");
    println!("'{}' 를 검색할 거에요 (최대 {}개)", args.query, args.limit);
}
