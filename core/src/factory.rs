#![allow(dead_code)]

use {
    rocket::{
        fairing::{
            Fairing,
            Info,
            Kind
        },
        Rocket,
    },
    std::{
        collections::BTreeMap,
        error::Error,
        path::Path,
        sync::{Arc, RwLock}
    },
    super::{
        configuration,
        constants,
        error,
        result
    }
};

fn is_file_handled(path: &Path) -> bool
{
    lazy_static! {
        static ref HANDLED_EXTENSIONS: [&'static std::ffi::OsStr; 3] = [
            std::ffi::OsStr::new("json"),
            std::ffi::OsStr::new("yml"),
            std::ffi::OsStr::new("yaml")
        ];
    }

    if !path.is_file() {
        return false;
    }

    match path.extension() {
        Some(extension) => {
            HANDLED_EXTENSIONS[..].contains(&extension)
        },
        _ => false
    }
}

#[derive(Clone, Debug, Default)]
pub struct Factory
{
    configurations: Arc<RwLock<BTreeMap<String, configuration::Configuration>>>,

    #[cfg(debug_assertions)] // If running development mode
    dev_configurations: Arc<RwLock<BTreeMap<String, configuration::Configuration>>>
}

impl Factory
{
    pub fn new() -> Self
    {
        Self {
            configurations: Arc::new(RwLock::new(BTreeMap::new())),

            #[cfg(debug_assertions)] // If running development mode
            dev_configurations: Arc::new(RwLock::new(BTreeMap::new()))
        }
    }

    fn load_directory(
        path: &Path,
        configurations_to_load: &RwLock<BTreeMap<String, configuration::Configuration>>
    )
        -> Result<(), error::Error>
    {
        for entry in path.read_dir().map_err(|err| error::Error::new(error::ErrorKind::Other, err.description()))? {
            let entry = entry.map_err(|err| error::Error::new(error::ErrorKind::Other, err.description()))?;
            let path = entry.path();

            if is_file_handled(&path) {
                if let Ok(mut configurations) = configurations_to_load.write() {
                    if let Some(_previous_value) = configurations.insert(
                        path.file_stem()
                            .expect("expected valid file name")
                            .to_str().ok_or_else(|| error::Error::new(error::ErrorKind::Other, "invalid file name"))?
                            .to_owned(),
                        {
                            eprintln!(
                                "Configuration file awaiting for initialization: {:?}",
                                path.file_name().unwrap_or(
                                    std::ffi::OsStr::new("invalid file name")
                                )
                            );

                            let configuration = configuration::Configuration::new(&path);
                            configuration.load()?;

                            eprintln!(
                                "Configuration file initialized: {:?}",
                                path.file_name().unwrap_or(
                                    std::ffi::OsStr::new("invalid file name")
                                )
                            );

                            configuration
                        }
                    ) {
                        return Err(error::Error::new(
                            error::ErrorKind::Other,
                            format!(
                                "a configuration already exists for '{}'",
                                path.file_stem()
                                    .expect("expected valid file name")
                                    .to_str()
                                    .ok_or_else(|| error::Error::new(
                                        error::ErrorKind::Other,
                                        "invalid file name"
                                    ))?
                            )
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(debug_assertions)] // If running development mode
    fn load_development_directory(&self)
        -> Result<(), error::Error>
    {
        Self::load_directory(
            &Path::new(constants::DEV_CONFIGURATION_DIRECTORY),
            &self.dev_configurations
        )
    }

    fn load_production_directory(&self)
        -> Result<(), error::Error>
    {
        Self::load_directory(
            &Path::new(constants::CONFIGURATION_DIRECTORY),
            &self.configurations
        )
    }

    pub fn load(&self)
        -> Result<(), error::Error>
    {
        self.load_production_directory()?;

        // If running development mode
        #[cfg(debug_assertions)] self.load_development_directory()?;

        Ok(())
    }

    #[cfg(debug_assertions)]
    fn get_development(&self, configuration_name: &str)
        -> result::Result<configuration::Configuration>
    {
        if let Ok(guard) = self.dev_configurations.read() {
            guard.get(configuration_name).ok_or_else(|| error::Error::from(
                error::ErrorKind::MissingValue
            )).map(|configuration: &'_ configuration::Configuration|
                (*configuration).clone()
            )
        }
        else {
            Err(error::Error::new(
                error::ErrorKind::Other, "dev_configurations got poisoned"
            ))
        }
    }

    pub fn get(&self, configuration_name: &str) -> result::Result<configuration::Configuration>
    {
        // First, try to get development configuration if compiled in development
        #[cfg(debug_assertions)]
        {
            if let Ok(configuration) = self.get_development(configuration_name) {
                return Ok(configuration);
            }
            // Error is ignored
        }

        // Then, if not available tries to return production configuration 
        if let Ok(guard) = self.configurations.read() {
            guard.get(configuration_name).ok_or_else(|| error::Error::from(
                error::ErrorKind::MissingValue
            )).map(|configuration: &'_ configuration::Configuration|
                (*configuration).clone()
            )
        }
        else {
            Err(error::Error::new(
                error::ErrorKind::Other, "configurations got poisoned"
            ))
        }
    }
}

impl Fairing for Factory
{
    fn info(&self) -> Info
    {
        Info {
            name: "Configuration factory",
            kind: Kind::Attach
        }
    }

    fn on_attach(&self, rocket: Rocket)
        -> std::result::Result<Rocket, Rocket>
    {
        // Loads available configurations
        let _ = self.load();

        // Stores himself in the state
        let rocket = rocket.manage((*self).clone());

        Ok(rocket)
    } 
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::OpenOptions;
    use std::io::Result;
    use std::io::Write as _;
    use std::path::{Path, PathBuf};
    use tempfile;

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

    #[test]
    fn is_file_handled()
    {
        assert_eq!(super::is_file_handled(Path::new("/unknown-file")), false);

        let file = create_temporary_file("", "", 24, &env::temp_dir()).unwrap();
        assert_eq!(super::is_file_handled(file.path()), false);
        delete_temporary_file(file);

        let file = create_temporary_file("", ".json", 24, &env::temp_dir()).unwrap();
        assert_eq!(super::is_file_handled(file.path()), true);
        delete_temporary_file(file);

        let file = create_temporary_file("", ".yml", 24, &env::temp_dir()).unwrap();
        assert_eq!(super::is_file_handled(file.path()), true);
        delete_temporary_file(file);

        let file = create_temporary_file("", ".yaml", 24, &env::temp_dir()).unwrap();
        assert_eq!(super::is_file_handled(file.path()), true);
        delete_temporary_file(file);
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

    #[test]
    fn load()
    {
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
            let factory = super::Factory::new();

            factory.load().expect("failed to load factory");

            let _config = factory.get("diesel")
                .expect("failed to get diesel configuration");
        }

        // Deletes temporary environment
        unmount_load_env(directories, files);

        // Comes back to initial dir
        let _ = cwd(&previous_dir);

        // Deletes temp dir
        delete_temporary_directory(temp_dir);
    }
}