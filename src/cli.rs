pub struct Config {
    pub option: String,
    pub source: String,
}

impl Config {
    pub fn build(env: Vec<String>) -> Config {
        let option = env[1].clone();
        let source = env[2].clone();

        Config { option, source }
    }
}
