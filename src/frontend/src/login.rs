use failure::Error;
use yew::prelude::*;
use yew::format::{Nothing, Json};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew_router::prelude::RouterButton;
use stdweb::traits::IEvent;
use stdweb::web::window;
use types::Person;

pub struct Login {
    updating: bool,
    persons: Option<Vec<Person>>,

    email: String,
    password: String,

    onlogin: Callback<()>,

    fetch_service: FetchService,
    link: ComponentLink<Login>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    FetchNew,
    FetchError,
    FetchReady(Result<Vec<Person>, Error>),
    UpdateEmail(String),
    UpdatePassword(String),
    Login
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[props(required)]
    pub onlogin: Callback<()>,
}


impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Login {
            updating: false,
            persons: None,

            email: "".to_string(),
            password: "".to_string(),

            onlogin: props.onlogin,

            fetch_service: FetchService::new(),
            link,
            ft: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchError => { }
            Msg::FetchNew => {
                self.updating = true;
                let task = {
                    let callback = self.link.send_back(
                        move |response: Response<Json<Result<Vec<Person>, Error>>>| {
                            let (meta, Json(data)) = response.into_parts();
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
            }
            Msg::FetchReady(persons) => {
                self.updating = false;
                self.persons = persons.ok();
            }
            Msg::UpdateEmail(email) => self.email = email,
            Msg::UpdatePassword(pw) => self.password = pw,
            Msg::Login => {
                self.onlogin.emit(());
                return true;
            }
        }
        true
    }
}

impl Renderable<Login> for Login {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="login-form-container">
                <div class="login-form-header">
                    <h5>{ "Log in" }</h5>
                </div>
                <div class="login-form">
                    <form>
                        <div class="form-group">
                            <label for="inputEmail">{ "Email address" }</label>
                            <input type="email" id="inputEmail" class="form-control" placeholder="Email address" required="" autofocus=""
                            value=&self.email oninput=|e| Msg::UpdateEmail(e.value) />
                        </div>

                        <div class="form-group">
                            <label for="inputPassword">{ "Password" }</label>
                            <input type="password" id="inputPassword" class="form-control" placeholder="Password" required=""
                            value=&self.password  oninput=|e| Msg::UpdatePassword(e.value) />
                        </div>

                        <button type="submit" class="btn btn-primary" onclick=|e| { e.prevent_default(); Msg::Login }>{ "Log in "}</button>
                    </form>
                </div>
            </div>
            /*<div class="container">
                <div class="person-list">
                    <button class="btn btn-primary" onclick=|_| Msg::FetchNew>{ "Fetch person list" }</button>
                    <div>
                        { self.render_persons() }
                    </div>
                </div>
            </div>*/
        }
    }
}

fn render_person(person: &Person) -> Html<Login> {
    html! {
        <li>{format!("Name: {}", person.name)}</li>
    }
}

impl Login {
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
