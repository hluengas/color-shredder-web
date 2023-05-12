use rand::Rng;
// needed to get the reference to HtmlCanvasElement
use wasm_bindgen::JsCast;

use gloo_timers::callback::Interval;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::{html, Component, Context, Html, NodeRef};

// yew messages
pub(crate) enum Msg {
    RenderCanvas,
    RandomizeCanvas,
}

// yew sub-component for an html canvas
pub(crate) struct Canvas {
    node_ref: NodeRef,
    image_data: Vec<u8>,
    height: u32,
    width: u32,
    _refresh_interval: Interval,
}
impl Canvas {
    // render logic
    fn render_canvas(&self) {
        // if node_ref can be cast as HtmlCanvasElement then render the canvas
        let Some(canvas) = self.node_ref.cast::<HtmlCanvasElement>() else { return };
        // get canvas context
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .expect("[Error] failed to get context of HtmlCanvasElement")
            .expect("[Error] failed to get context of HtmlCanvasElement")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("[Error] js-sys dynamic cast failed");

        context.set_image_smoothing_enabled(false);

        let width: usize = self.width as usize;
        let height: usize = self.height as usize;

        // Convert the RGB buffer to RGBA
        let rgba_data: Vec<u8> = self
            .image_data
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
    fn randomize_canvas(&mut self) {
        let width: usize = self.width as usize;
        let height: usize = self.height as usize;
        let mut rng: rand::rngs::ThreadRng = rand::thread_rng();

        // Create a test pattern RGB image buffer
        self.image_data = vec![0u8; width * height * 3];
        for y in 0..height {
            for x in 0..width {
                let random_u8: u8 = rng.gen();
                let offset: usize = (y * width + x) * 3;
                self.image_data[offset] = random_u8;
                self.image_data[offset + 1] = random_u8;
                self.image_data[offset + 2] = random_u8;
            }
        }
    }
}
impl Component for Canvas {
    type Message = Msg;
    type Properties = ();

    // Canvas init
    fn create(ctx: &Context<Self>) -> Self {
        let interval: Interval = {
            let link = ctx.link().clone();
            Interval::new(1000 / 25, move || link.send_message(Msg::RenderCanvas))
        };
        let blank_image_buffer: Vec<u8> = vec![1u8; 100 * 100 * 3];
        Self {
            node_ref: NodeRef::default(),
            image_data: blank_image_buffer,
            height: 100u32,
            width: 100u32,
            _refresh_interval: interval,
        }
    }

    // Canvas update logic
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::RenderCanvas => {
                self.render_canvas();
            }
            Msg::RandomizeCanvas => {
                self.randomize_canvas();
            }
        }
        false
    }

    // Canvas view logic
    fn view(&self, ctx: &Context<Self>) -> Html {
        let random_button_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::RandomizeCanvas);

        html! {
            <div>
                <div class="centered-button">
                    <button onclick={random_button_callback}>
                        { "Generate Image" }
                    </button>
                </div>
                <div class="centered-canvas">
                    <canvas ref={self.node_ref.clone()} width={self.width.to_string()} height={self.height.to_string()}></canvas>
                </div>
            </div>
        }
    }
}
