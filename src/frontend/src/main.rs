#![recursion_limit="1024"]

#[cfg_attr(test, macro_use)]
extern crate stdweb;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate web_logger;

mod api;
mod login;
mod router;
mod forum;

pub fn main() {
    web_logger::init();
    info!("Starting app");
    yew::start_app::<router::Model>();
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_thing() {
        stdweb::initialize();
        js! {
            console.log("aoentusdaeou")
        };
        assert_eq!(1 + 1, 2);
    }
}
