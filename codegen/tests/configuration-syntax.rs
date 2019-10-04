extern crate rocket_config;
#[macro_use] extern crate rocket_config_codegen;

configuration!("diesel");

// This just checks that the DieselConfiguration struct exists
#[test]
fn test_valid() {
    let _diesel = DieselConfiguration(
        rocket_config::Configuration::new(
            std::path::Path::new("/tmp/diesel.json")
        )
    );
}