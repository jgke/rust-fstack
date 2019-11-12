use yew::prelude::*;
use crate::login::Login;
use crate::forum::Forum;

use yew::virtual_dom::VNode;
use yew_router::{route::Route, service::RouteService, Switch};


#[derive(Clone, Switch, Debug)]
pub enum AppRoute {
    #[to = "/#forum"]
    Forum,
    #[to = "/"]
    Login,
}

pub struct Model {
    route_service: RouteService<()>,
    route: Route<()>,
    token: Option<String>,

    link: ComponentLink<Self>
}

pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
    Login(String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        let route = Route::from(route);
        let callback = link.send_back(|(route, state)| -> Msg {
            Msg::RouteChanged(Route {
                route,
                state: Some(state),
            })
        });
        route_service.register_callback(callback);
        Model {
            route_service,
            route,
            link,
            token: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => self.route = route,
            Msg::Login(token) => {
                self.token = Some(token);
                self.link.send_self(Msg::ChangeRoute(AppRoute::Forum));
            }
            Msg::ChangeRoute(route) => {
                // This might be derived in the future
                let route_string = match route {
                    AppRoute::Login => "/",
                    AppRoute::Forum => "/#forum"
                };
                self.route_service.set_route(&route_string, ());
                self.route = Route {
                    route: route_string.to_string(),
                    state: None,
                };
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> VNode<Self> {
        html! {
            match (AppRoute::switch(self.route.clone()), &self.token) {
                (Some(AppRoute::Login), _) | (_, None) => html!{<Login onlogin=|token| Msg::Login(token)/>},
                (Some(AppRoute::Forum), Some(token)) => {
                    if let Some(token) = &self.token {
                        html!{<Forum token=token.to_string()/>}
                    } else {
                        unreachable!()
                    }
                }
                (None, _) => html!{"404"}
            }
        }
    }
}
