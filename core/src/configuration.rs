#![allow(dead_code)]

use {
    std::{
        error::Error,
        io::{self, Read},
        path::{Path, PathBuf},
        sync::{Arc, RwLock}
    },
    crate::{
        error,
        result,
        value::{Index, Value}
    }
};

#[derive(Clone, Debug)]
pub struct Configuration
{
    configuration:  Arc<RwLock<Option<Value>>>,
    path:           Arc<RwLock<PathBuf>>,
}

impl Configuration
{
    pub fn new(path: &Path) -> Self
    {
        Self {
            configuration:  Arc::new(RwLock::new(None)),
            path:           Arc::new(RwLock::new(path.to_owned())),
        }
    }

    fn apply_to_configuration<T, F>(&self, f: F) -> result::Result<T>
    where F: Fn(&RwLock<Option<Value>>) -> result::Result<T>
    {
        f(&self.configuration)
    }

    pub fn is_loaded(&self) -> result::Result<bool>
    {
        self.apply_to_configuration(
            |configuration: &RwLock<Option<Value>>| {
                if let Ok(guard) = configuration.read() {
                    Ok(guard.is_some())
                }
                else {
                    Err(error::Error::new(
                        error::ErrorKind::Other, "configuration got poisoned"
                    ))
                }
            }
        )
    }

    fn read_file(&self) -> Result<String, io::Error>
    {
        if let Ok(path) = self.path.read() {
            std::fs::File::open(path.clone())
            .and_then(|mut file: std::fs::File| -> Result<String, io::Error> {
                let mut content = String::new();

                // TODO: Removes the use of read_to_string for the profit of a
                // safer read method (handling non-utf8 characters)
                match file.read_to_string(&mut content) {
                    Ok(_size) => { Ok(content) },
                    Err(err) => { Err(err) }
                }
            })
        }
        else {
            Err(io::Error::new(
                io::ErrorKind::Other, "path got poisoned"
            ))
        }
    }

    fn deserialize(&self, extension: &str, content: String)
        -> Result<(), error::Error>
    {
        let deserialized;

        match extension {
            "json"          => {
                let deserialized_json = serde_json::from_str::<serde_json::Value>(content.as_ref())
                .map_err(|err| error::Error::new(
                        error::ErrorKind::Other, err.description()
                    )
                )?;

                deserialized = Value::from(&deserialized_json);
            },
            "yml" | "yaml"  => {
                let deserialized_yaml = serde_yaml::from_str::<serde_yaml::Value>(content.as_ref())
                .map_err(|err| error::Error::new(
                        error::ErrorKind::Other, err.description()
                    )
                )?;

                deserialized = Value::from(&deserialized_yaml);
            },
            format          => {
                return Err(error::Error::new(
                    error::ErrorKind::UnimplementedFormat,
                    format!("unimplemented format: {}", format)
                ));
            }
        };

        if let Ok(mut configuration) = self.configuration.write() {
            (*configuration) = Some(deserialized);
            Ok(())
        }
        else {
            Err(error::Error::new(
                error::ErrorKind::Other,
                "configuration got poisoned"
            ))
        }
    }

    pub fn load(&self) -> Result<(), error::Error>
    {
        // First, check if already loaded
        match self.is_loaded()
        {
            // If it returns an error, forwards
            Err(err)  => return Err(err),

            // If it's loaded, returns immediate value
            Ok(loaded) if loaded => return Ok(()),

            // Continue
            _ => {}
        }

        // Then, if it is not, load it (this will be async when available)
        if let Ok(path) = self.path.read() {
            let ext: &str = match path.extension().ok_or_else(|| error::Error::new(
                error::ErrorKind::MissingValue, "no extension available"
            )).and_then(|ext| {
                if let Some(ext) = ext.to_str() { Ok(ext) }
                else {
                    Err(error::Error::new(
                        error::ErrorKind::FormatError,
                        "extension's format is invalid"
                    ))
                }
            }) {
                Ok(ext) => ext,
                Err(err) => {
                    return Err(err);
                }
            };

            let content = match self.read_file().map_err(|err| {
                error::Error::new(error::ErrorKind::MissingValue, err.description())
            }) {
                Ok(ext) => ext,
                Err(err) => { return Err(err); }
            };

            self.deserialize(ext, content)
        }
        else {
            Err(error::Error::new(
                error::ErrorKind::Other, "path got poisoned"
            ))
        }
    }

    pub fn get<I: Index>(&self, index: I) -> result::Result<Option<Value>>
    {
        let _ = self.load();

        if let Ok(configuration) = self.configuration.read() {
            Ok({
                if let Some(ref_configuration) = configuration.as_ref() {
                    match ref_configuration.get(index) {
                        Some(value) => Some(value.clone()),
                        None => None
                    }
                }
                else { None }
            })
        }
        else {
            Err(error::Error::new(
                error::ErrorKind::Other, "configuration got poisoned"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs::OpenOptions;
    use std::io::Write;
    #[cfg(any(unix, target_os = "redox"))] use std::os::unix::ffi::OsStrExt;
    #[cfg(windows)] use std::os::windows::prelude::*;
    use tempfile;

    #[test]
    fn configuration() {
        let configuration = Configuration::new(&Path::new("/random-path"));

        assert_eq!(configuration.is_loaded().unwrap(), false);
        assert!(configuration.get("invalid_index").unwrap().is_none());
    }

    #[test]
    fn missing_extension() {
        let temp_file = tempfile::NamedTempFile::new()
            .expect("failed to create a named temp file");

        {
            let mut file = OpenOptions::new()
                .write(true)
                .open(temp_file.path())
                .expect(&format!("failed to open {:?}", temp_file.path()));
            let _ = file.write(b"This is a test content");
        }

        let configuration = Configuration::new(temp_file.path());
        let err = configuration.load().expect_err("expected an Err, got a result");

        assert_eq!(err.kind(), error::ErrorKind::MissingValue);
        assert_eq!(err.description(), "no extension available");
    }

    #[test]
    fn invalid_extension() {
        #[cfg(any(unix, target_os = "redox"))]
        let temp_file = tempfile::Builder::new()
            .prefix("")
            .suffix(OsStr::from_bytes(b".te\x80st"))
            .rand_bytes(16)
            .tempfile()
            .expect("failed to create a named temp file");
        
        #[cfg(windows)]
        let temp_file = tempfile::Builder::new()
            .prefix("")
            .suffix(OsString::from_wide(b".te\x80st").as_os_str())
            .rand_bytes(16)
            .tempfile()
            .expect("failed to create a named temp file");

        {
            let mut file = OpenOptions::new()
                .write(true)
                .open(temp_file.path())
                .expect(&format!("failed to open {:?}", temp_file.path()));
            let _ = file.write(b"This is a test content");
        }

        let configuration = Configuration::new(temp_file.path());
        let err = configuration.load().expect_err("expected an Err, got a result");

        assert_eq!(err.kind(), error::ErrorKind::FormatError);
        assert_eq!(err.description(), "extension's format is invalid");
    }

    #[test]
    fn unimplemented_extension() {
        let temp_file = tempfile::Builder::new()
            .prefix("")
            .suffix(".unimp")
            .rand_bytes(16)
            .tempfile()
            .expect("failed to create a named temp file");

        {
            let mut file = OpenOptions::new()
                .write(true)
                .open(temp_file.path())
                .expect(&format!("failed to open {:?}", temp_file.path()));
            let _ = file.write(b"This is a test content");
        }

        let configuration = Configuration::new(temp_file.path());
        let err = configuration.load().expect_err("expected an Err, got a result");

        assert_eq!(err.kind(), error::ErrorKind::UnimplementedFormat);
        assert_eq!(err.description(), "unimplemented format: unimp");
    }

    #[test]
    fn valid_json() {
        let temp_file = tempfile::Builder::new()
            .prefix("test")
            .suffix(".json")
            .rand_bytes(8)
            .tempfile()
            .expect("failed to create a named temp file");

        {
            let mut dot_json = OpenOptions::new()
                .write(true)
                .open(temp_file.path())
                .expect("failed to open testXXXXXXXX.json");
            let _ = dot_json
                .write(&serde_json::to_vec(&json!({
                    "parameters": {
                        "env(DATABASE_URL)": "",
                        "inital_id": 0,
                        "limit_id": -1,
                    },
                }
            )).expect("failed to serialize example json")[..]);
        }

        let configuration = Configuration::new(temp_file.path());
        let _ = configuration.load().expect("expected to load config");

        let parameters = configuration.get("parameters");
        assert!(parameters.is_ok());
        let parameters = parameters.unwrap();
        assert!(parameters.is_some());
        let parameters = parameters.unwrap();
        assert!(parameters.is_object());

        assert!(parameters.get("env(DATABASE_URL)").is_some());
        assert_eq!(parameters.get("env(DATABASE_URL)").unwrap().as_str().unwrap(), "");
    }

    #[test]
    fn valid_yaml() {
        let temp_file = tempfile::Builder::new()
            .prefix("test")
            .suffix(".yaml")
            .rand_bytes(8)
            .tempfile()
            .expect("failed to create a named temp file");

        {
            let mut dot_json = OpenOptions::new()
                .write(true)
                .open(temp_file.path())
                .expect("failed to open testXXXXXXXX.yaml");
            let _ = dot_json
                .write(b"parameters:\n    env(DATABASE_URL): 'test'");
        }

        let configuration = Configuration::new(temp_file.path());
        let _ = configuration.load().expect("expected to load config");

        let parameters = configuration.get("parameters");
        assert!(parameters.is_ok());
        let parameters = parameters.unwrap();
        assert!(parameters.is_some());
        let parameters = parameters.unwrap();
        assert!(parameters.is_object());

        assert!(parameters.get("env(DATABASE_URL)").is_some());
        assert_eq!(parameters.get("env(DATABASE_URL)").unwrap().as_str().unwrap(), "test");
    }
}