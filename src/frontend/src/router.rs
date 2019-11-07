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
}

pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
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
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => self.route = route,
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
            <div>
            {
                info!("{:?}", &self.route);
                match AppRoute::switch(self.route.clone()) {
                    Some(AppRoute::Login) => html!{<Login onlogin=|_| Msg::ChangeRoute(AppRoute::Forum)/>},
                    Some(AppRoute::Forum) => html!{<Forum />},
                    None => html!{"404"}
                }
            }
            </div>
        }
    }
}
