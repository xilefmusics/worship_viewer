mod group;
mod user;

pub use group::{Group, GroupDatabase};
pub use user::{User, UserDatabase};

pub trait IdGetter {
    fn get_id_first(&self) -> String;
    fn get_id_second(&self) -> String;
    fn get_id_full(&self) -> String;
}
