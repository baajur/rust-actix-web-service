#[macro_use]
//mod macros;
//mod crypto;
//mod music_api;
//mod request;
mod server;
mod http;
use crate::server::{start_server, Opt};
use structopt::StructOpt;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    start_server(&opt).await
}
