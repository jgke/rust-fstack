use failure::Error;
use yew::prelude::*;
use yew::format::{Nothing, Json};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew_router::prelude::RouterButton;
use stdweb::traits::IEvent;
use stdweb::web::window;

pub struct Login {
    email: String,
    password: String,

    onlogin: Callback<()>,

    fetch_service: FetchService,
    link: ComponentLink<Login>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    UpdateEmail(String),
    UpdatePassword(String),
    Login,
    CreateAccount,
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
            Msg::UpdateEmail(email) => self.email = email,
            Msg::UpdatePassword(pw) => self.password = pw,
            Msg::Login => {
                self.onlogin.emit(());
                return true;
            }
            Msg::CreateAccount => {
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
                <div class="login-form-content">
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

                            <div class="login-buttons">
                                <button type="submit" class="btn btn-primary" onclick=|e| { e.prevent_default(); Msg::Login }>{ "Log in "}</button>
                                <button type="submit" class="btn btn-link" onclick=|e| { e.prevent_default(); Msg::CreateAccount }>{ "Create account"}</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        }
    }
}
