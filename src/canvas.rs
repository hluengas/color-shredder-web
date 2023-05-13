use rand::Rng;
// needed to get the reference to HtmlCanvasElement
use wasm_bindgen::JsCast;

use gloo_timers::callback::Interval;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::{html, Component, Context, Html, NodeRef};

// yew messages
pub(crate) enum Msg {
    RenderCanvas,
    ResetCanvas,
    RandomizeCanvas,
    TogglePixel(i32, i32),
}

#[derive(Copy, Clone)]
pub(crate) struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    _boundry_val: u8,
}

// yew sub-component for an html canvas
pub(crate) struct Canvas {
    node_ref: NodeRef,
    image_data: Vec<Pixel>,
    height: u32,
    width: u32,
    _pixels_placed_count: u64,
    _boundry_pixels: Vec<Pixel>,
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
            .iter()
            .flat_map(|pixel| vec![pixel.red, pixel.red, pixel.red, 255u8])
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
        self.image_data = vec![
            Pixel {
                red: 0u8,
                green: 0u8,
                blue: 0u8,
                _boundry_val: 0u8
            };
            width * height
        ];
        for pixel in self.image_data.iter_mut() {
            let random_u8: u8 = rng.gen();
            pixel.red = random_u8;
            pixel.green = random_u8;
            pixel.blue = random_u8;
        }
    }
    fn toggle_pixel(&mut self, x_coord: i32, y_coord: i32) {
        let x_index: usize = x_coord
            .try_into()
            .expect("[Error] got mouse click position outside of canvas");
        let y_index: usize = y_coord
            .try_into()
            .expect("[Error] got mouse click position outside of canvas");
        self.image_data[get_linear_index(x_index, y_index, self.width as usize)] = Pixel {
            red: 0u8,
            green: 0u8,
            blue: 0u8,
            _boundry_val: 0u8,
        }
    }
    fn reset_canvas(&mut self) {
        let width: usize = self.width as usize;
        let height: usize = self.height as usize;

        // Create a test pattern RGB image buffer
        self.image_data = vec![
            Pixel {
                red: 255u8,
                green: 255u8,
                blue: 255u8,
                _boundry_val: 0u8
            };
            width * height
        ];
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
        let blank_image_buffer: Vec<Pixel> = vec![
            Pixel {
                red: 255u8,
                green: 255u8,
                blue: 255u8,
                _boundry_val: 0u8
            };
            100 * 100
        ];
        Self {
            node_ref: NodeRef::default(),
            image_data: blank_image_buffer,
            height: 100u32,
            width: 100u32,
            _pixels_placed_count: 0u64,
            _boundry_pixels: Vec::new(),
            _refresh_interval: interval,
        }
    }

    // Canvas update logic
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ResetCanvas => {
                self.reset_canvas();
            }
            Msg::RenderCanvas => {
                self.render_canvas();
            }
            Msg::RandomizeCanvas => {
                self.randomize_canvas();
            }
            Msg::TogglePixel(x_coord, y_coord) => {
                self.toggle_pixel(x_coord, y_coord);
            }
        }
        false
    }

    // Canvas view logic
    fn view(&self, ctx: &Context<Self>) -> Html {
        let reset_button_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::ResetCanvas);
        let random_button_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::RandomizeCanvas);
        let canvas_mouse_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_event: web_sys::MouseEvent| {
                Msg::TogglePixel(_event.offset_x(), _event.offset_y())
            });

        html! {
            <div>
                <div class="centered-button">
                    <button onclick={reset_button_callback}>
                        { "Reset Image" }
                    </button>
                </div>
                <div class="centered-button">
                    <button onclick={random_button_callback}>
                        { "Generate Image" }
                    </button>
                </div>
                <div class="centered-canvas">
                    <canvas
                        ref={self.node_ref.clone()}
                        width={self.width.to_string()}
                        height={self.height.to_string()}
                        onclick={canvas_mouse_callback}
                    ></canvas>
                </div>
            </div>
        }
    }
}

fn get_linear_index(x: usize, y: usize, width: usize) -> usize {
    (y * width) + x
}
