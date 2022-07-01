pub struct Packet {
    pub packet: Vec<u32>,
}

pub mod api;
pub mod config;
pub mod constants;
pub mod model;
pub mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
