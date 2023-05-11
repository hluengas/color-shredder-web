// needed to get the reference to HtmlCanvasElement
use wasm_bindgen::JsCast;

use rand::Rng;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::{html, Component, Context, Html, NodeRef};

// yew messages
pub(crate) enum Msg {
    RenderCanvas,
}

// yew sub-component for an html canvas
pub(crate) struct HtmlCanvasComponent {
    canvas_node_ref: NodeRef,
}
impl HtmlCanvasComponent {
    // render logic
    fn render_canvas(&self) {
        // if canvas_node_ref can be cast as HtmlCanvasElement then render the canvas
        if let Some(canvas) = self.canvas_node_ref.cast::<HtmlCanvasElement>() {
            // get canvas context
            let context: CanvasRenderingContext2d = canvas
                .get_context("2d")
                .expect("[Error] failed to get context of HtmlCanvasElement")
                .expect("[Error] failed to get context of HtmlCanvasElement")
                .dyn_into::<CanvasRenderingContext2d>()
                .expect("[Error] js-sys dynamic cast failed");

            context.set_image_smoothing_enabled(false);

            let width: usize = canvas.width() as usize;
            let height: usize = canvas.height() as usize;

            // Create a test pattern RGB image buffer
            let mut rng: rand::rngs::ThreadRng = rand::thread_rng();
            let mut image_data: Vec<u8> = vec![0u8; width * height * 3];
            for y in 0..height {
                for x in 0..width {
                    let random_u8: u8 = rng.gen();
                    let offset: usize = (y * width + x) * 3;
                    image_data[offset] = random_u8;
                    image_data[offset + 1] = random_u8;
                    image_data[offset + 2] = random_u8;
                }
            }

            // Convert the RGB buffer to RGBA
            let rgba_data: Vec<u8> = image_data
                .chunks(3)
                .flat_map(|rgb| vec![rgb[0], rgb[1], rgb[2], 255])
                .collect();

            // convert framebuffer into js-sys ImageData object
            let image_data: ImageData = ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(&rgba_data),
                width as u32,
                height as u32,
            )
            .expect("[Error] to create js-sys iamge data");

            // write the ImageData to the html canvas contex
            context
                .put_image_data(&image_data, 0.0, 0.0)
                .expect("[Error] js-sys failed to write image data to canvas context");
        }
    }
}
impl Component for HtmlCanvasComponent {
    type Message = Msg;
    type Properties = ();

    // HtmlCanvasComponent init
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_node_ref: NodeRef::default(),
        }
    }

    // HtmlCanvasComponent update logic
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RenderCanvas => {
                self.render_canvas();
            }
        }
        false
    }

    // HtmlCanvasComponent view logic
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div class="centered-button">
                    <button onclick={ctx.link().callback(|_| Msg::RenderCanvas)}>
                        { "Render Test Pattern" }
                    </button>
                </div>
                <div class="centered-canvas">
                    <canvas ref={self.canvas_node_ref.clone()} width="800" height="400"></canvas>
                </div>
            </div>
        }
    }
}
