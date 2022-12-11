
//Generate two configs with duplicate fields, but one's fields are all optional.
//In doing so, you can call ".apply_optional()" on the real config to bring in the
//"optionally set" values from the other config. This is all wrapped up if you
//just read a chain from the filesystem (you provide the chain to .read_chain())
macro_rules! create_config {
    ($configname:ident, $optconfigname:ident => {
        $($name:ident : $type:ty, )*
    }) => {
        //The standard config, you most likely want to be using THIS one
        #[derive(serde::Deserialize, Clone, Default, Debug)]
        pub struct $configname
        {
            $(pub $name: $type,)*
        }

        //The "optional" config, this is used internally
        #[derive(serde::Deserialize, Clone, Debug)]
        struct $optconfigname
        {
            $(
                #[serde(default)]
                pub $name: Option<$type>,
            )*
        }

        impl $configname {
            //Mutate ourselves to apply the non-empty fields from the optional config. This has to be in the macro...
            fn apply_optional(&mut self, opt: $optconfigname) {
                $(
                    if let Some(item) = opt.$name { self.$name = item; }
                )*
            }
            //* Even though a trait would be better for these next two functions, it's just WAY easier to put them
            //  in the configuration itself (at least for now)

            //This creates a filled out configuration by applying the given toml files (if they exist) one after
            //another in the order given. It starts with purely default values. It does not throw errors on 
            //files not existing
            pub fn read_chain_toml(chain: Vec<String>) -> Self {
                let mut result = Self::default(); 

                for filename in chain {
                    //Maybe async someday? idk. Also reading into memory? It's just configs so it's fine
                    //but clearly there are much better ways (serde gives from_reader)
                    let data = std::fs::read_to_string(filename);
                    match data {
                        Ok(data) => {
                            let config_result: Result<$optconfigname, _> = toml::from_str(&data);
                            match config_result {
                                Ok(config) => {
                                    result.apply_optional(config);
                                }
                                Err(error) => {
                                    println!("read_chain_json json parse error: {}", error.to_string())
                                }
                            }
                        }
                        Err(error) => {
                            println!("read_chain_json file read error: {}", error.to_string())
                        }
                    }
                }

                result
            }
            //The basic case of "I just want to load settings for the given environment". If you give
            //(settings, Dev), it will read from the chain "settings.json, settings.Dev.json"
            pub fn read_with_environment_toml(basename: &str, env: Option<&str>) -> Self {
                let mut chain = vec![ format!("./{}.toml", basename) ];
                if let Some(env) = env {
                    chain.push(format!("./{}.{}.toml", basename, env));
                }
                Self::read_chain_toml(chain)
            }
        }
    };
}
pub(crate) use create_config;
