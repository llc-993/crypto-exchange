pub use block_common::{common, wallet, transaction};

pub fn init() {
    block_common::init();
    println!("初始化区块链模块...");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}