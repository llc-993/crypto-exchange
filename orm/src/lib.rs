pub mod models;
pub mod repositories;
pub mod entities;
pub mod migrations;

pub fn init() {
    println!("初始化ORM模块...");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}