use adjectives::ADJECTIVES;
use animals::ANIMALS;

mod adjectives;
mod english;
mod animals;

pub fn random_name() -> String {
    let adjective = fastrand::choice(ADJECTIVES).unwrap();
    let animal = fastrand::choice(ANIMALS).unwrap();
    format!("{adjective}-{animal}")
}

#[macro_export]
macro_rules! b {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => break Err(err.into()),
        }
    };
}