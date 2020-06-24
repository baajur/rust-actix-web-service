use actix_web::{
    HttpServer,
    App,
    web,
    HttpResponse,
    get,
    middleware,
};
use structopt::StructOpt;
use listenfd::ListenFd;
use std::env;
use dotenv::dotenv;
//use crate::music_api::*;
use crate::bll::*;
use futures::executor;
use std::{sync::mpsc, thread};
use mysql::*;
use mysql::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "RustNeteaseCloudMusicApi", about = "A Netease Cloud Music Rust API Service")]
pub(crate) struct Opt {
    #[structopt(long, default_value = "0.0.0.0")]
    ip: String,

    #[structopt(short, long, default_value = "8000")]
    port: i32
}

#[get("/hello")]
async fn hello() -> HttpResponse {
   HttpResponse::Ok().body("hello!")
}

#[get("/stop")]
async fn stop(stopper: web::Data<mpsc::Sender<()>>) -> HttpResponse {
    // make request that sends message through the Sender
    stopper.send(()).unwrap();

    HttpResponse::NoContent().finish()
}

// systemfd --no-pid -s http::8000 -- cargo watch -x run
// cargo run
// cargo update
// cargo build --release
// sudo nohup ./target/release/rust-actix-web-service &
pub(crate) async fn start_server(opt: &Opt) -> std::io::Result<()> {
      dotenv().ok();

      //std::env::set_var("RUST_LOG", "actix_server=debug,actix_web=debug");
      //env_logger::init();

      // create a channel
      let (tx, rx) = mpsc::channel::<()>();

      // mysql
      let url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
      let pool = mysql::Pool::new(url).unwrap();

      // start server as normal but don't .await after .run() yet
      let mut listenfd = ListenFd::from_env();
      let mut server = HttpServer::new(move || {
            App::new()
            .data(tx.clone())
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            //.service(web::resource("/get").route(web::get().to(list)))
            .service(list)
            .service(hello)
            .service(stop)
            .service(
                actix_files::Files::new("/", "./public/").index_file("index.html")
            )
      });

      let env = env::var("ENV").expect("ENV is not set in .env file");
      if env == "dev" {
            server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
                  server.listen(l)?
            } else {
                  server.bind(format!("{}:{}", opt.ip, opt.port))?
            };
            server
            .run()
            .await
      } else {
            let server = server
            .bind(format!("{}:{}", opt.ip, opt.port))?
            .run();
            // clone the Server handle
            let srv = server.clone();
            thread::spawn(move || {
                // wait for shutdown signal
                rx.recv().unwrap();

                // stop server gracefully
                executor::block_on(srv.stop(true))
            });

            // run server
            server.await
      }
}