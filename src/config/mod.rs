use std::env::var;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub port: u16,
}

impl Config {
    pub fn init()  -> Config {
        let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_maxage = var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");

        Config {
            database_url,
            jwt_secret,
            jwt_expiration: jwt_maxage.parse::<i64>().unwrap(),
            port: 8080,
        }
    }
}