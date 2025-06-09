use dotenvy::dotenv;
use std::env;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ConfigError {
    pub missing_vars: Vec<String>,
    pub invalid_vars: Vec<(String, String)>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Configuration validation failed:")?;
        
        if !self.missing_vars.is_empty() {
            writeln!(f, "Missing required environment variables:")?;
            for var in &self.missing_vars {
                writeln!(f, "  - {}", var)?;
            }
        }
        
        if !self.invalid_vars.is_empty() {
            writeln!(f, "Invalid environment variables:")?;
            for (var, reason) in &self.invalid_vars {
                writeln!(f, "  - {}: {}", var, reason)?;
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for ConfigError {}

#[derive(Clone, Debug)]
pub struct EnvConfig {
    vars: HashMap<String, String>,
}

impl EnvConfig {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
    
    pub fn get_parsed<T>(&self, key: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + 'static,
    {
        self.vars.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key).into())
            .and_then(|v| v.parse::<T>().map_err(|e| e.into()))
    }
}

pub fn validate_env_vars(required_vars: &[&str]) -> Result<EnvConfig, ConfigError> {
    dotenv().ok();
    
    let mut missing_vars = Vec::new();
    let mut vars = HashMap::new();
    
    for &var_name in required_vars {
        match env::var(var_name) {
            Ok(val) if !val.trim().is_empty() => {
                vars.insert(var_name.to_string(), val);
            }
            Ok(_) => {
                missing_vars.push(format!("{} (empty)", var_name));
            }
            Err(_) => {
                missing_vars.push(var_name.to_string());
            }
        }
    }
    
    if missing_vars.is_empty() {
        Ok(EnvConfig { vars })
    } else {
        Err(ConfigError {
            missing_vars,
            invalid_vars: Vec::new(),
        })
    }
}

// Macro for easy validation
#[macro_export]
macro_rules! validate_env {
    ($($var:expr),+ $(,)?) => {{
        $crate::validate_env_vars(&[$($var),+])
    }};
}

/* Example usage:
mod env_validator;

use env_validator::EnvValidator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = validate_env![
        "DATABASE_URL",
        "PORT",
        "API_KEY",
        "LOG_LEVEL",
        "MAX_CONNECTIONS"
    ]?;
    
    // Use get_parsed with explicit type annotation
    let port: u16 = config.get_parsed("PORT")?;
    let max_connections: u32 = config.get_parsed("MAX_CONNECTIONS")?;
    
    // Or use turbofish syntax
    let port = config.get_parsed::<u16>("PORT")?;
    
    // Get raw string values
    let database_url = config.get("DATABASE_URL").unwrap();
    let api_key = config.get("API_KEY").unwrap();
    
    println!("Server starting on port {}", port);
    println!("Max connections: {}", max_connections);
    
    Ok(())
}
*/