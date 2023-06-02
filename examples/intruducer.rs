use intruducer::{intruduce, Error};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "intruducer")]
struct Opt {
    /// Target process id
    #[structopt()]
    id: u32,

    /// Library path
    #[structopt(short, long, parse(from_os_str))]
    lib_path: PathBuf,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    intruduce(opt.id, opt.lib_path)?;

    println!("Successful intruduction!");

    Ok(())
}
