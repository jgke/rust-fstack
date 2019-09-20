#[cfg(cargo_web)]
mod entry {
    extern crate frontend;

    pub fn main() {
        frontend::start();
    }
}

#[cfg(not(cargo_web))]
mod entry {
    extern crate backend;

    pub fn main() {
        backend::start();
    }
}

fn main() {
    entry::main();
}
