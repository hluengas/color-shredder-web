mod app;
mod canvas;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
