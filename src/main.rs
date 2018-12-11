use futures::Future;
use rust_trending_discord::*;
use std::env::args;

fn main() -> Result<(), Error> {
    openssl_probe::init_ssl_cert_env_vars();

    let args: Vec<_> = args().collect();
    let config = if args.len() >= 2 {
        Config::from_file(&args[1])?
    } else {
        Config::from_file("config.toml")?
    };

    let bot = RustTrending::from_config(config)?;
    tokio::run(bot.run_loop().map_err(|e| panic!("{}", e)));

    Ok(())
}
