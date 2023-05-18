use rand::random;
// needed to get the reference to HtmlCanvasElement
use wasm_bindgen::JsCast;

use gloo_timers::callback::Interval;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, ImageData, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};
use yew::{html, Component, Context, Html, NodeRef};
use yew_bootstrap::component::Button;
use yew_bootstrap::util::{include_cdn, include_cdn_js, Color};

// yew messages
pub(crate) enum Msg {
    RenderCanvas,
    ResetCanvas,
    FitCanvas,
    RandomizeCanvas,
    TogglePixel(i32, i32),
    ZoomIn,
    ZoomOut,
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
    view_height: u32,
    width: u32,
    view_width: u32,
    pixels_placed_count: u64,
    boundry_pixels: Vec<Pixel>,
    _refresh_interval: Interval,
}
impl Canvas {
    // render logic
    fn render_canvas(&self) {
        // if node_ref can be cast as HtmlCanvasElement then render the canvas
        let Some(canvas_ref) = self.node_ref.cast::<HtmlCanvasElement>() else { return };

        // if node_ref can be cast as HtmlCanvasElement then render the canvas
        let offscreen_canvas: OffscreenCanvas =
            OffscreenCanvas::new(self.width, self.height).unwrap();

        // get canvas context
        let canvas_2d: CanvasRenderingContext2d = canvas_ref
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        // get canvas context
        let offscreen_canvas_2d: OffscreenCanvasRenderingContext2d = offscreen_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .unwrap();

        canvas_2d.set_image_smoothing_enabled(false);
        offscreen_canvas_2d.set_image_smoothing_enabled(false);

        // Convert the RGB buffer to RGBA
        let rgba_data: Vec<u8> = self
            .image_data
            .iter()
            .flat_map(|pixel| vec![pixel.red, pixel.green, pixel.blue, 255u8])
            .collect();

        // convert framebuffer into js-sys ImageData object
        let image_data: ImageData = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba_data),
            self.width as u32,
            self.height as u32,
        )
        .unwrap();

        // write the ImageData to the html canvas contex
        offscreen_canvas_2d
            .put_image_data(&image_data, 0.0, 0.0)
            .unwrap();

        canvas_2d
            .draw_image_with_offscreen_canvas_and_dw_and_dh(
                &offscreen_canvas,
                0.0,
                0.0,
                self.view_width as f64,
                self.view_height as f64,
            )
            .unwrap();
    }
    fn randomize_canvas(&mut self) {
        // Create a test pattern RGB image buffer
        self.image_data = vec![
            Pixel {
                red: 0u8,
                green: 0u8,
                blue: 0u8,
                _boundry_val: 0u8
            };
            self.width as usize * self.height as usize
        ];
        for pixel in self.image_data.iter_mut() {
            pixel.red = random::<u8>();
            pixel.green = random::<u8>();
            pixel.blue = random::<u8>();
        }
    }
    fn toggle_pixel(&mut self, view_x_coord: i32, view_y_coord: i32) {
        let view_scale: f64 = (self.view_width / self.width) as f64;

        let x_index: usize = (view_x_coord as f64 * (1.0f64 / view_scale)).trunc() as usize;
        let y_index: usize = (view_y_coord as f64 * (1.0f64 / view_scale)).trunc() as usize;

        let linear_index = get_linear_index(x_index, y_index, self.width as usize);

        let inverted_red: u8 = 255u8 - self.image_data[linear_index].red;
        let inverted_green: u8 = 255u8 - self.image_data[linear_index].green;
        let inverted_blue: u8 = 255u8 - self.image_data[linear_index].blue;

        self.image_data[linear_index] = Pixel {
            red: inverted_red,
            green: inverted_green,
            blue: inverted_blue,
            _boundry_val: 0u8,
        }
    }
    fn reset_canvas(&mut self) {
        let width: usize = self.width as usize;
        let height: usize = self.height as usize;

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
    }
    fn zoom_in_canvas(&mut self) {
        self.view_width = self.view_width * 2u32;
        self.view_height = self.view_height * 2u32;

        // if node_ref can be cast as HtmlCanvasElement then render the canvas
        let Some(canvas_ref) = self.node_ref.cast::<HtmlCanvasElement>() else { return };

        canvas_ref.set_width(self.view_width);
        canvas_ref.set_height(self.view_height);
    }
    fn zoom_out_canvas(&mut self) {
        if self.view_width > self.width {
            self.view_width = self.view_width / 2u32;
            self.view_height = self.view_height / 2u32;

            // if node_ref can be cast as HtmlCanvasElement then render the canvas
            let Some(canvas_ref) = self.node_ref.cast::<HtmlCanvasElement>() else { return };

            canvas_ref.set_width(self.view_width);
            canvas_ref.set_height(self.view_height);
        }
    }
    fn fit_canvas_to_screen(&mut self) {
        // get window & screen from web-sys
        let window = web_sys::window().unwrap();

        let width_margin = 32u32;
        let height_margin = 256u32;

        // get the height & width from the screen
        let canvas_width =
            (window.inner_width().unwrap().as_f64().unwrap() as u32 - width_margin) / 4u32;
        let canvas_height =
            (window.inner_height().unwrap().as_f64().unwrap() as u32 - height_margin) / 4u32;
        let canvas_view_width = canvas_width * 4u32;
        let canvas_view_height = canvas_height * 4u32;

        // generate blank image data
        let blank_image_buffer: Vec<Pixel> = vec![
            Pixel {
                red: 0u8,
                green: 0u8,
                blue: 0u8,
                _boundry_val: 0u8
            };
            (canvas_height * canvas_width) as usize
        ];

        // get the canvas ref and alter the canvas's size
        let Some(canvas_ref) = self.node_ref.cast::<HtmlCanvasElement>() else { return };
        canvas_ref.set_width(canvas_view_width);
        canvas_ref.set_height(canvas_view_height);

        // make all other needed state changes
        self.image_data = blank_image_buffer;
        self.height = canvas_height;
        self.view_height = canvas_view_height;
        self.width = canvas_width;
        self.view_width = canvas_view_width;
        self.pixels_placed_count = 0u64;
        self.boundry_pixels = Vec::new();
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
        // get window & screen from web-sys
        let window = web_sys::window().unwrap();
        let width_margin = 32u32;
        let height_margin = 256u32;

        // get the height & width from the screen
        let canvas_width =
            (window.inner_width().unwrap().as_f64().unwrap() as u32 - width_margin) / 4u32;
        let canvas_height =
            (window.inner_height().unwrap().as_f64().unwrap() as u32 - height_margin) / 4u32;
        let canvas_view_width = canvas_width * 4u32;
        let canvas_view_height = canvas_height * 4u32;

        // generate blank image data
        let blank_image_buffer: Vec<Pixel> = vec![
            Pixel {
                red: 0u8,
                green: 0u8,
                blue: 0u8,
                _boundry_val: 0u8
            };
            (canvas_height * canvas_width) as usize
        ];

        // make all other needed state changes
        Self {
            node_ref: NodeRef::default(),
            image_data: blank_image_buffer,
            height: canvas_height,
            view_height: canvas_view_height,
            width: canvas_width,
            view_width: canvas_view_width,
            pixels_placed_count: 0u64,
            boundry_pixels: Vec::new(),
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
            Msg::FitCanvas => {
                self.fit_canvas_to_screen();
            }
            Msg::ZoomIn => {
                self.zoom_in_canvas();
            }
            Msg::ZoomOut => {
                self.zoom_out_canvas();
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
        let fit_canvas_to_screen: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::FitCanvas);
        let random_button_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::RandomizeCanvas);
        let zoom_in_button_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::ZoomIn);
        let zoom_out_button_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|_| Msg::ZoomOut);
        let canvas_mouse_callback: yew::Callback<web_sys::MouseEvent> =
            ctx.link().callback(|event: web_sys::MouseEvent| {
                Msg::TogglePixel(event.offset_x(), event.offset_y())
            });

        html! {
            <div class="dark_container">
                {include_cdn()}
                <div class="centered-button">
                    <Button onclick={reset_button_callback} style={Color::Dark} text={"Reset Image"} />
                    <Button onclick={random_button_callback} style={Color::Dark} text={"Generate Image"} />
                    <Button onclick={fit_canvas_to_screen} style={Color::Dark} text={"Fit to Screen"} />
                </div>
                <div class="centered-button">
                    <Button onclick={zoom_in_button_callback} style={Color::Dark} text={"Zoom In"} />
                    <Button onclick={zoom_out_button_callback} style={Color::Dark} text={"Zoom Out"} />
                </div>
                <h />
                <div class="centered-canvas">
                    <canvas
                        width={self.view_width.to_string()}
                        height={self.view_height.to_string()}
                        ref={self.node_ref.clone()}
                        onclick={canvas_mouse_callback}
                    ></canvas>
                </div>
                {include_cdn_js()}
            </div>
        }
    }
}

fn get_linear_index(x: usize, y: usize, width: usize) -> usize {
    (y * width) + x
}
