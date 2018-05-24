pub mod c_nereon;

#[derive(Debug)]
enum CfgData {
    Int(i64),
    Bool(bool),
    String(String),
    Array(Vec<Cfg>),
    IpPort(i32),
    Float(f32),
    Object(Vec<Cfg>),
}

#[derive(Debug)]
pub struct Meta {}

#[derive(Debug)]
pub struct Cfg {
    key: String,
    data: CfgData,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
