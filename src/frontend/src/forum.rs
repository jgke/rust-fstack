use failure::Error;
use yew::prelude::*;
use yew::format::{Nothing, Json};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew_router::prelude::RouterButton;
use stdweb::traits::IEvent;
use stdweb::web::window;
use types::Thread;

pub struct Forum {
    updating: bool,
    threads: Option<Vec<Thread>>,
    current_thread: Option<Thread>,

    fetch_service: FetchService,
    link: ComponentLink<Forum>,
    ft: Option<FetchTask>,
}

pub enum Msg {
    Fetch,
    FetchError,
    FetchReady(Result<Vec<Thread>, Error>),
}

impl Component for Forum {
    type Message = Msg;
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Forum {
            updating: false,
            threads: None,
            current_thread: None,

            fetch_service: FetchService::new(),
            link,
            ft: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchError => { }
            Msg::Fetch => {
                self.updating = true;
                let task = {
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
                        let request = Request::get("http://localhost:80/threads").body(Nothing).unwrap();
                        self.fetch_service.fetch(request, callback)
                };
                self.ft = Some(task);
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
                            <button class="btn btn-primary">{"Create thread"}</button>
                        </div>
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
        <li>{format!("Name: {}", thread.title)}</li>
    }
}

impl Forum {
    fn render_threads(&self) -> Html<Self> {
        if let Some(threads) = &self.threads {
            html! {
                <ul>
                    { for threads.iter().map(render_thread) }
                </ul>
            }
        } else {
            html! {
                <p class="p-3"> { "No threads available. How about creating one?" } </p>
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
}
