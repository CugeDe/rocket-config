# rocket-config

rocket-config is a [Fairing](https://api.rocket.rs/v0.4/rocket/fairing/trait.Fairing.html)
designed for Rocket, a web framework for Rust (nightly).

```rust
#![feature(proc_macro_hygiene)]

#[macro_use] extern crate rocket;
extern crate rocket_config;
#[macro_use] extern crate rocket_config_codegen;

// This will generate the DieselConfiguration struct
// used below.
configuration!("diesel");

use rocket_config::Factory as ConfigurationsFairing;

// Here, `_configuration` contains the parsed configuration
// file "diesel.{json,yml,yaml}"
#[get("/<name>/<age>")]
fn hello(_configuration: DieselConfiguration, name: String, age: u8)
-> String
{
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    rocket::ignite()
        .attach(ConfigurationsFairing::new())
        .mount("/hello", routes![hello]).launch();
}
```