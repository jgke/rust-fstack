#[macro_use]
extern crate log;
extern crate web_logger;

use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

struct Model { }

enum Msg {
    DoIt,
}

impl Component for Model {
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model { }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DoIt => {
                info!("Doing it!");
                // Update your model on events
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            // Render your model here
            <button onclick=|_| Msg::DoIt>{ "Click me!" }</button>
        }
    }
}

pub fn start() {
    web_logger::init();
    info!("Starting app");
    yew::start_app::<Model>();
}
