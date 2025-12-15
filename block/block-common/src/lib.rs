pub mod common;
pub mod wallet;
pub mod transaction;

pub fn init() {
    println!("初始化区块链公共模块...");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}