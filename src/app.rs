use crate::canvas::HtmlCanvasComponent;
use yew::{html, Component, Context, Html};

// yew component for root App
// app state -empty-
pub(crate) struct App;
impl Component for App {
    type Message = ();
    type Properties = ();

    // app init
    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    // app update logic
    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    // app view logic
    fn view(&self, _ctx: &Context<Self>) -> Html {
        // render HtmlCanvasComponent sub-compononent
        html! {
            <HtmlCanvasComponent />
        }
    }
}
