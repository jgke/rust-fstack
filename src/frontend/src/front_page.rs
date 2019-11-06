use failure::Error;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::format::{Nothing, Json};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use types::Person;

pub struct FrontPage {
    updating: bool,
    persons: Option<Vec<Person>>,

    fetch_service: FetchService,
    link: ComponentLink<FrontPage>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    FetchNew,
    FetchError,
    FetchReady(Result<Vec<Person>, Error>),
}

impl Component for FrontPage {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        FrontPage {
            updating: false,
            persons: None,

            fetch_service: FetchService::new(),
            link,
            ft: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchError => { false }
            Msg::FetchNew => {
                self.updating = true;
                let task = {
                    let callback = self.link.send_back(
                        move |response: Response<Json<Result<Vec<Person>, Error>>>| {
                            let (meta, Json(data)) = response.into_parts();
                            info!("META: {:?}, {:?}", &meta, &data);
                            if meta.status.is_success() {
                                Msg::FetchReady(data)
                            } else {
                                Msg::FetchError
                            }
                        },
                        );
                        let request = Request::get("http://localhost:80/").body(Nothing).unwrap();
                        self.fetch_service.fetch(request, callback)
                };
                self.ft = Some(task);
                true
            }
            Msg::FetchReady(persons) => {
                self.updating = false;
                self.persons = persons.ok();
                true
            }
        }
    }
}

impl Renderable<FrontPage> for FrontPage {
    fn view(&self) -> Html<Self> {
        html! {
            //<div class="container">
                <div class="person-list">
                    <button class="btn btn-primary" onclick=|_| Msg::FetchNew>{ "Fetch person list" }</button>
                    <div>
                        { self.render_persons() }
                    </div>
                </div>
            //</div>
        }
    }
}

fn render_person(person: &Person) -> Html<FrontPage> {
    html! {
        <li>{format!("Name: {}", person.name)}</li>
    }
}

impl FrontPage {
    fn render_persons(&self) -> Html<Self> {
        if let Some(persons) = &self.persons {
            html! {
                <ul>
                    { for persons.iter().map(render_person) }
                </ul>
            }
        } else {
            html! {
                <p> { "No persons." } </p>
            }
        }
    }
}
