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