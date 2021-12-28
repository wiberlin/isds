#![allow(clippy::wildcard_imports)]
#![macro_use]
pub use gloo;
pub use gloo::console::log;
use gloo::render::{request_animation_frame, AnimationFrame};

pub use yew;
use yew::prelude::*;

mod components;
pub use components::*;

mod protocols;
pub use protocols::*;

mod simulation;
pub use simulation::*;

pub struct Isds {
    pub sim: SharedSimulation,
    last_render: RealSeconds,
    _render_loop_handle: Option<AnimationFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsdsContext {
    pub sim: SharedSimulation,
    pub last_render: RealSeconds,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Rendered(RealSeconds),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub sim: SharedSimulation,
}

impl Component for Isds {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let sim = ctx.props().sim.clone();

        // We do this to make sure that any `do_now` things are done before the children ISDS
        // components get initialized.
        sim.borrow_mut().catch_up(0.);

        Self {
            sim,
            last_render: 0.,
            _render_loop_handle: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            < ContextProvider<IsdsContext> context={ IsdsContext { sim: self.sim.clone(), last_render: self.last_render }}>
                { for ctx.props().children.iter() }
            </ ContextProvider<IsdsContext>>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(time) => {
                let elapsed_browser_seconds = time - self.last_render;
                self.sim.borrow_mut().catch_up(elapsed_browser_seconds);
                self.last_render = time;
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        // if first_render { // TODO ? or rather something for a UI component ?
        //     let window = gloo::utils::window();
        //     window.add_event_listener_with_callback(
        //         "onkeydown",
        //         ctx.link().callback(move |e: KeyboardEvent| { e.prevent_default(); Msg::KeyDown(e) })
        //     );
        // }

        // code inspired by yew's webgl example
        let handle = {
            let link = ctx.link().clone();
            request_animation_frame(move |time| link.send_message(Msg::Rendered(time / 1000.)))
        };
        // A reference to the handle must be stored, otherwise it is dropped and the render won't
        // occur.
        self._render_loop_handle = Some(handle);
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn tests_work() {
        assert!(true);
    }
}