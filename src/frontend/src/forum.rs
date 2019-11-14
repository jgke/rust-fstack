use failure::Error;
use yew::prelude::*;
use yew::format::{Nothing, Json};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use types::{CreateMessage, CreateThread, Message, Thread};

use crate::api;

pub struct Forum {
    updating: bool,
    threads: Option<Vec<Thread>>,
    current_thread: Option<Thread>,
    show_create_thread: bool,
    create_thread_field: String,
    create_message_field: String,

    token: String,

    fetch_service: FetchService,
    link: ComponentLink<Forum>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    FetchThreads,
    FetchError,

    CreateThreadForm,
    CreateThread,
    UpdateCreateTitle(String),

    ChooseThread(i32),

    ThreadsFetched(Result<Vec<Thread>, Error>),
    ThreadFetched(Result<Thread, Error>),

    UpdateMessageField(String),
    CreateMessage(i32),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[props(required)]
    pub token: String,
}

impl Component for Forum {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut this = Forum {
            updating: false,
            threads: None,
            current_thread: None,
            show_create_thread: false,
            create_thread_field: "".to_string(),
            create_message_field: "".to_string(),

            token: props.token,

            fetch_service: FetchService::new(),
            link,
            ft: None,
        };
        this.ft = Some(this.fetch_threads());
        this
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchError => { }
            Msg::FetchThreads => {
                self.updating = true;
                self.ft = Some(self.fetch_threads());
            }
            Msg::CreateThreadForm => {
                self.show_create_thread = true;
                self.create_thread_field = "".to_string();
            }
            Msg::CreateThread => {
                self.show_create_thread = false;
                self.ft = Some(self.create_thread())
            }
            Msg::UpdateCreateTitle(s) => {
                self.create_thread_field = s;
            }
            Msg::UpdateMessageField(s) => {
                self.create_message_field = s;
            }
            Msg::CreateMessage(thread_id) => {
                self.ft = Some(self.create_message(thread_id));
            }
            Msg::ChooseThread(id) => {
                self.ft = Some(self.choose_thread(id));
            }
            Msg::ThreadsFetched(threads) => {
                self.updating = false;
                self.threads = threads.ok();
            }
            Msg::ThreadFetched(thread) => {
                self.updating = false;
                self.current_thread = thread.ok();
            }
        }
        true
    }
}

impl Renderable<Forum> for Forum {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="forum-view">
                <div class="row">
                    <div class="thread-list">
                        <div class="thread-list-header">
                            <h5>{ "Thread list" }</h5>
                            {
                                if !self.show_create_thread {
                                    html!{ <button
                                            class="btn btn-primary"
                                            onclick=|_| Msg::CreateThreadForm>{"Create thread"}</button> }
                                } else {
                                    html!{}
                                }
                            }
                        </div>
                        { self.create_thread_form() }
                        <div class="thread-list-content">
                            { self.render_threads() }
                        </div>
                    </div>
                    <div class="thread-view">
                        { self.render_current_thread() }
                    </div>
                </div>
            </div>
        }
    }
}

impl Forum {
    fn render_threads(&self) -> Html<Self> {
        if let Some(threads) = &self.threads {
            html! {
                <div class="list-group">
                    { for threads.iter().map(|t| self.render_thread(t)) }
                </div>
            }
        } else {
            html! {
                <p class="p-3"> { "Loading threads..." } </p>
            }
        }
    }

    fn render_current_thread(&self) -> Html<Self> {
        if let Some(thread) = &self.current_thread {
            html! {
                <div class="thread">
                    <h4>{ &thread.title }</h4>
                    { self.render_current_messages(&thread.messages.as_ref().unwrap_or(&vec![])) }
                    <hr />
                    { self.create_message_field() }
                </div>
            }
        } else {
            html! {
                <div class="no-thread-selected">
                    <p> { "Choose a thread to get started!" } </p>
                </div>
            }
        }
    }

    fn render_current_messages(&self, messages: &[Message]) -> Html<Self> {
        if messages.is_empty() {
            html! {
                "No messages yet! Be the first one to post here ;)"
            }
        } else {
            html! {
                <ul class="list-group">
                { for messages.iter().map(
                        |msg| html! {
                            <li class="list-group-item">{ format!("{} | {}", &msg.creator, &msg.content) }</li>
                        })
                }
                </ul>
            }
        }
    }

    fn create_message_field(&self) -> Html<Self> {
        if let Some(thread) = &self.current_thread {
            let id = thread.id;
            html! {
                <form>
                    <div class="form-group">
                        <label for="inputMessage">{ "Message" }</label>
                        <input id="inputMessage" class="form-control" placeholder="Create new message"
                        autofocus="" autocomplete="off"
                        value=&self.create_message_field oninput=|e| Msg::UpdateMessageField(e.value) />
                    </div>

                    <button class="btn btn-primary" onclick=|_| Msg::CreateMessage(id)>{ "Send message" }</button>
                </form>
            }
        } else {
            html!{}
        }
    }

    fn fetch_threads(&mut self) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Vec<Thread>, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::ThreadsFetched(data)
                } else {
                    Msg::FetchError
                }
            },
        );
        let request = Request::get(api::all_threads()).body(Nothing).unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn choose_thread(&mut self, id: i32) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Thread, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::ThreadFetched(data)
                } else {
                    Msg::FetchError
                }
            },
        );
        let request = Request::get(api::thread(id)).body(Nothing).unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn create_thread(&mut self) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Thread, Error>>>| {
                let (meta, Json(_)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::FetchThreads
                } else {
                    Msg::FetchError
                }
            },
        );
        let body = CreateThread { title: self.create_thread_field.to_string() };

        let request = Request::post(api::new_thread())
            .header("token", &self.token)
            .body(Ok(serde_json::to_string(&body).unwrap()))
            .unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn create_message(&mut self, thread_id: i32) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<(), Error>>>| {
                let (meta, Json(_)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::ChooseThread(thread_id)
                } else {
                    Msg::FetchError
                }
            },
        );
        let body = CreateMessage { content: self.create_message_field.to_string() };
        let request = Request::post(api::new_message(thread_id))
            .header("token", &self.token)
            .body(Ok(serde_json::to_string(&body).unwrap()))
            .unwrap();
        self.create_message_field = "".to_string();
        self.fetch_service.fetch(request, callback)
    }

    fn create_thread_form(&self) -> Html<Self> {
        if self.show_create_thread {
            html! {
                <form class="create-thread">
                    <div class="form-group">
                        // Killer feature: comments!
                        <label for="inputTitle">{ "Title" }</label>
                        <input id="inputTitle" class="form-control" placeholder="Thread title" required=""
                        autofocus="" autocomplete="off"
                        value=&self.create_thread_field oninput=|e| Msg::UpdateCreateTitle(e.value) />
                    </div>

                    <button class="btn btn-primary" onclick=|_| Msg::CreateThread>{ "Create thread" }</button>
                </form>
            }
        } else {
            html!{}
        }
    }

    fn render_thread(&self, thread: &Thread) -> Html<Forum> {
        let id = thread.id;
        if Some(id) == self.current_thread.as_ref().map(|t| t.id) {
            html! { <button class="thread-list-item active disabled">
                <b>{ &thread.title }</b>
                <br />
                <small>{ &thread.creator }</small>
            </button> }
        } else {
            html! { <button class="thread-list-item" onclick=|_| Msg::ChooseThread(id)>
                <b>{ &thread.title }</b>
                <br />
                <small>{ &thread.creator }</small>
            </button> }
        }
    }
}
