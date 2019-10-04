#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_config;
#[macro_use] extern crate serde_json;
extern crate tempfile;

use rocket::local::Client;
use rocket_config::Factory as ConfigurationsFairing;

use std::env;
use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write as _;
use std::path::{Path, PathBuf};

configuration!("diesel");

fn create_temporary_file(prefix: &str, suffix: &str, rand_bytes: usize, dest: &Path)
    -> Result<tempfile::NamedTempFile>
{
    tempfile::Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .rand_bytes(rand_bytes)
        .tempfile_in(dest)
}

fn delete_temporary_file(temp_file: tempfile::NamedTempFile)
{
    let _ = temp_file.close();
}

fn create_temporary_directory(prefix: &str, suffix: &str, rand_bytes: usize, dest: &Path)
    -> Result<tempfile::TempDir>
{
    tempfile::Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .rand_bytes(rand_bytes)
        .tempdir_in(dest)
}

fn delete_temporary_directory(temp_dir: tempfile::TempDir)
{
    let _ = temp_dir.close();
}

fn cwd(path: &Path) -> PathBuf
{
    let current_dir = env::current_dir()
        .expect("failed to retrieve current directory");

    env::set_current_dir(path)
        .expect("failed to change current directory");
    current_dir
}

fn mount_load_env(path: &Path)
    -> (Vec<tempfile::TempDir>, Vec<tempfile::NamedTempFile>)
{
    let mut directories = Vec::new();
    let mut files = Vec::new();

    // Create the following directories tree:
    // .
    // └── config
    //     └── dev
    {
        directories.push(
            create_temporary_directory("config", "", 0, path).unwrap()
        );

        directories.push(
            create_temporary_directory("dev", "", 0, directories[0].path()).unwrap()
        );
    }

    // Create the following final tree:
    // .
    // └── config
    //     └── dieselXXXXXXXX.json          # Valid file
    //     └── no_extension                 # Invalid file (no ext.)
    //     └── dev
    //         └── dieselXXXXXXXX.json      # Valid file
    //         └── invalid_extension.toto   # Invalid file (unhandled ext.)
    {
        // Creates an invalid file in production directory
        files.push(
            create_temporary_file("no_extension", "", 16, directories[0].path()).unwrap()
        );

        // Creates an invalid file in development directory
        files.push(
            create_temporary_file("invalid_extension_dev", "toto", 4, directories[1].path()).unwrap()
        );

        // Creates a valid file in production directory
        {
            files.push(
                create_temporary_file("diesel", ".json", 0, directories[0].path()).unwrap()
            );

            let mut diesel_dot_json = OpenOptions::new()
                .write(true)
                .open(files.last().unwrap().path())
                .expect("failed to open diesel.json");
            let _ = diesel_dot_json
                .write(&serde_json::to_vec(&json!({
                    "parameters": {
                        "env(DATABASE_URL)": "",
                        "inital_id": 0,
                        "limit_id": -1,
                    },
                    "diesel": {
                        "dbal": {
                            "driver": "mysql",
                            "server_version": 5.7,
                            "charset": "utf8",
                            "default_table_options": {
                                "charset": "utf8",
                                "collate": "utf8_unicode_ci"
                            },
                            "url": "%env(resolve:DATABASE_URL)%"
                        }
                    }
                }
            )).expect("failed to serialize example json")[..]);
        }

        // Creates a valid file in development directory
        {
            files.push(
                create_temporary_file("diesel", ".json", 0, directories[1].path()).unwrap()
            );

            let mut diesel_dot_json = OpenOptions::new()
                .write(true)
                .open(files.last().unwrap().path())
                .expect("failed to open diesel.json");
            let _ = diesel_dot_json
                .write(&serde_json::to_vec(&json!({
                    "parameters": {
                        "env(DATABASE_URL)": "",
                        "inital_id": 0,
                        "limit_id": -1,
                    },
                    "diesel": {
                        "dbal": {
                            "driver": "mysql",
                            "server_version": 5.7,
                            "charset": "utf8",
                            "default_table_options": {
                                "charset": "utf8",
                                "collate": "utf8_unicode_ci"
                            },
                            "url": "%env(resolve:DATABASE_URL)%"
                        }
                    }
                }
            )).expect("failed to serialize example json")[..]);
        }
    }

    (directories, files)
}

fn unmount_load_env(directories: Vec<tempfile::TempDir>, files: Vec<tempfile::NamedTempFile>)
{
    // Deletes all files
    for file in files {
        delete_temporary_file(file);
    }

    // Deletes all dirs
    for directory in directories {
        delete_temporary_directory(directory);
    }
}

#[get("/<name>/<age>")]
fn hello(configuration: DieselConfiguration, name: String, age: u8) -> String {
    println!("Configuration: {:?}", configuration);

    format!("Hello, {} year old named {}!", age, name)
}

#[test]
fn rocket_test() {
    // Creates temporary environment
    let temp_dir = tempfile::tempdir().expect(
        &format!("failed to create temp dir in {:?}", env::temp_dir())
    );

    // Creates temporary environment
    let (directories, files) = mount_load_env(temp_dir.path());

    // Moves to temporary environment
    let previous_dir = cwd(temp_dir.path());

    // Real logic
    {
        let rocket = rocket::ignite()
            .attach(ConfigurationsFairing::new())
            .mount("/hello", routes![hello]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let req = client.get("/hello/John%20Doe/37");
        let mut response = req.dispatch();
        let body = response.body_string();

        assert!(body.is_some());
        assert_eq!(body.unwrap(), "Hello, 37 year old named John Doe!");
    }

    // Deletes temporary environment
    unmount_load_env(directories, files);

    // Comes back to initial dir
    let _ = cwd(&previous_dir);

    // Deletes temp dir
    delete_temporary_directory(temp_dir);
}