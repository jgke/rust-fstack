use failure::Error;
use yew::prelude::*;
use yew::format::Json;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use stdweb::traits::IEvent;

use types::{CreateAccount, Token};

pub struct Login {
    email: String,
    password: String,
    error: Option<LoginError>,

    onlogin: Callback<String>,

    fetch_service: FetchService,
    link: ComponentLink<Login>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    UpdateEmail(String),
    UpdatePassword(String),
    Login,
    CreateAccount,
    FetchError(LoginError),
    LoginSuccess(String),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[props(required)]
    pub onlogin: Callback<String>,
}

pub enum LoginError {
    Login,
    CreateAccount,
}


impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Login {
            email: "".to_string(),
            password: "".to_string(),
            error: None,

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
                self.error = None;
                self.ft = Some(self.login());
                return true;
            }
            Msg::CreateAccount => {
                self.error = None;
                self.ft = Some(self.create_account());
                return true;
            }
            Msg::LoginSuccess(token) => {
                self.onlogin.emit(token);
            }
            Msg::FetchError(error) => {
                self.error = Some(error);
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

                            { self.login_error() }
                        </form>
                    </div>
                </div>
            </div>
        }
    }
}

impl Login {
    fn create_account(&mut self) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Token, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    if let Ok(token) = data {
                        Msg::LoginSuccess(token.token)
                    } else {
                        Msg::FetchError(LoginError::CreateAccount)
                    }
                } else {
                    Msg::FetchError(LoginError::CreateAccount)
                }
            },
        );

        let username = self.email.to_string();
        let password = self.password.to_string();

        let body = CreateAccount { username, password };

        let request = Request::post("http://localhost:80/account")
            .body(Ok(serde_json::to_string(&body).unwrap()))
            .unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn login(&mut self) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Token, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    if let Ok(token) = data {
                        Msg::LoginSuccess(token.token)
                    } else {
                        Msg::FetchError(LoginError::Login)
                    }
                } else {
                    Msg::FetchError(LoginError::Login)
                }
            },
        );

        let username = self.email.to_string();
        let password = self.password.to_string();

        let body = types::Login { username, password };

        let request = Request::post("http://localhost:80/login")
            .body(Ok(serde_json::to_string(&body).unwrap()))
            .unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn login_error(&self) -> Html<Self> {
        match self.error {
            Some(LoginError::Login) => html! {
                <div class="login-error">{ "Login failed, check username and password" }</div>
            },
            Some(LoginError::CreateAccount) => html! {
                <div class="login-error">{ "Account creation failed" }</div>
            },
            None => html! {}
        }
    }
}
