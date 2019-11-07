use failure::Error;
use yew::prelude::*;
use yew::format::{Nothing, Json};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use stdweb::traits::IEvent;
use types::Thread;
use serde_json::json;

pub struct Forum {
    updating: bool,
    threads: Option<Vec<Thread>>,
    current_thread: Option<Thread>,
    show_create_thread: bool,
    create_thread_title: String,

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

    ChooseThread(u32),

    FetchReady(Result<Vec<Thread>, Error>),
}

impl Component for Forum {
    type Message = Msg;
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut this = Forum {
            updating: false,
            threads: None,
            current_thread: None,
            show_create_thread: false,
            create_thread_title: "".to_string(),

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
                self.create_thread_title = "".to_string();
            }
            Msg::CreateThread => {
                self.show_create_thread = false;
                self.ft = Some(self.create_thread())
            }
            Msg::UpdateCreateTitle(s) => {
                self.create_thread_title = s;
            }
            Msg::ChooseThread(id) => {
            }
            Msg::FetchReady(threads) => {
                self.updating = false;
                self.threads = threads.ok();
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
                            { if !self.show_create_thread {
                                html!{ <button class="btn btn-primary" onclick=|_| Msg::CreateThreadForm>{"Create thread"}</button> }
                                                         } else { html!{} } }
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

fn render_thread(thread: &Thread) -> Html<Forum> {
    html! {
        <li class="list-group-item">{ &thread.title }</li>
    }
}

impl Forum {
    fn render_threads(&self) -> Html<Self> {
        if let Some(threads) = &self.threads {
            html! {
                <ul class="list-group">
                    { for threads.iter().map(render_thread) }
                </ul>
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
                "Foo"
            }
        } else {
            html! {
                <div class="no-thread-selected">
                    <p> { "Choose a thread to get started!" } </p>
                </div>
            }
        }
    }

    fn fetch_threads(&mut self) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Vec<Thread>, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::FetchReady(data)
                } else {
                    Msg::FetchError
                }
            },
        );
        let request = Request::get("http://localhost:80/thread").body(Nothing).unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn create_thread(&mut self) -> FetchTask {
        let callback = self.link.send_back(
            move |response: Response<Json<Result<Thread, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::FetchThreads
                } else {
                    Msg::FetchError
                }
            },
        );
        let title = &self.create_thread_title;
        let body = Json(&json!({"title": &self.create_thread_title}));
        let request = Request::post("http://localhost:80/thread")
            .body(Ok(format!("{{\"title\": \"{}\"}}", title)))
            .unwrap();
        self.fetch_service.fetch(request, callback)
    }

    fn create_thread_form(&self) -> Html<Self> {
        if self.show_create_thread {
            html! {
                <form class="create-thread">
                    <div class="form-group">
                        <label for="inputTitle">{ "Title" }</label>
                        <input id="inputTitle" class="form-control" placeholder="Thread title" required="" autofocus=""
                        value=&self.create_thread_title oninput=|e| Msg::UpdateCreateTitle(e.value) />
                    </div>

                    <button class="btn btn-primary" onclick=|_| Msg::CreateThread>{ "Create thread" }</button>
                </form>
            }
        } else {
            html!{}
        }
    }
}
