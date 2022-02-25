use crate::{
    components::{collapsible::Collapsible, sort_controls::SortControls, sort_graph::SortGraph},
    utils::{gen_u32_vec, knuth_shuffle},
};
use sorting_algorithms::*;
use std::num::ParseIntError;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SortingAlgorithm<T: Clone + Copy + PartialEq + PartialOrd> {
    pub name: &'static str,
    pub sort: fn(Vec<T>) -> SortResult<T>,
}

pub const SORTING_ALGORITHMS: [SortingAlgorithm<u32>; 3] = [
    SortingAlgorithm {
        name: "Bubble sort",
        sort: bubble_sort::sort,
    },
    SortingAlgorithm {
        name: "Insertion sort",
        sort: insertion_sort::sort,
    },
    SortingAlgorithm {
        name: "Merge sort",
        sort: merge_sort::sort,
    },
];

pub enum Msg {
    UpdateInput(Vec<u32>),
    /// Receives a new config and a boolean that controls if the change causes a rerender.
    UpdateConfig(SortConfig, bool),
    ChangeActiveStep(Result<usize, ParseIntError>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SortConfig {
    pub input_len: usize,
    pub sorting_algorithm: SortingAlgorithm<u32>,
    pub audio_enabled: bool,
}
impl Default for SortConfig {
    fn default() -> Self {
        Self {
            input_len: 100,
            sorting_algorithm: SORTING_ALGORITHMS[0].clone(),
            audio_enabled: true,
        }
    }
}

pub struct SortingAlgorithms {
    input: Vec<u32>,
    output: SortResult<u32>,
    sort_config: SortConfig,
    steps: Vec<Vec<SortCommand<u32>>>,
    active_step_index: usize,
}

impl Component for SortingAlgorithms {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let sort_config = SortConfig::default();
        let input = knuth_shuffle(gen_u32_vec(sort_config.input_len));
        let output = (sort_config.sorting_algorithm.sort)(input.clone());
        let active_step = output.steps.len() - 1;
        SortingAlgorithms {
            input,
            output: SortResult::new(output.output, output.duration, output.steps.clone()),
            sort_config,
            steps: output.steps,
            active_step_index: active_step,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateInput(val) => {
                self.input = val;
                self.update_values();
                true
            }
            Msg::UpdateConfig(val, rerender) => {
                self.sort_config = val;
                if rerender {
                self.update_values();
                }
                rerender
            }
            Msg::ChangeActiveStep(res) => {
                if let Ok(val) = res {
                    self.active_step_index = val;
                    return true;
                }
                false
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let active_step = (&self.steps[0..=self.active_step_index]).to_vec();
        let mut active_step_output = self.input.clone();
        run_sort_steps(&mut active_step_output, active_step);

        let sort_duration = format!(
            "{:?} ms",
            match &self.output.duration {
                Some(dur) => dur.as_millis(),
                None => 0,
            }
        );
        let change_active_step = ctx.link().callback(|e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            Msg::ChangeActiveStep(el.value().parse::<usize>())
        });
        let update_input = ctx.link().callback(Msg::UpdateInput);
        let update_config = ctx
            .link()
            .callback(|msg: (SortConfig, bool)| Msg::UpdateConfig(msg.0, msg.1));

        html! {
            <div id="SortingAlgorithms">
                <h1>{"Sorting algorithms"}</h1>

                <SortControls config={self.sort_config.clone()} {update_input} {update_config} />

                <div class="content">
                    <div class="input-container">
                <h2>{"Input"}</h2>

                        <Collapsible open={true} title={"Input graph"}>
                            <SortGraph items={self.input.clone()} />
                        </Collapsible>
                    </div>

                    <div class="output-container">
                        <h2>{ format!("Output ({} steps, {})", self.steps.len() - 1, sort_duration) }</h2>

                    <Collapsible open={true} title={"Output graph"}>
                                <SortGraph items={active_step_output} step={self.steps[self.active_step_index].clone()} audio_enabled={self.sort_config.audio_enabled} />
                    </Collapsible>

                        <div class="step-selector">
                            <label for="active-step-input">
                                { format!("Step: {}", self.active_step_index) }
                            </label>
                            <input type="range" id="active-step-input" min="0" max={(self.steps.len() - 1).to_string()} value={self.active_step_index.to_string()} oninput={change_active_step} />
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

impl SortingAlgorithms {
    fn update_values(&mut self) {
        self.input = knuth_shuffle(gen_u32_vec(self.sort_config.input_len));
        let output = (self.sort_config.sorting_algorithm.sort)(self.input.clone());
        self.output = SortResult::new(output.output, output.duration, output.steps.clone());
        self.steps = output.steps;

        if self.active_step_index >= self.steps.len() {
            self.active_step_index = self.steps.len() - 1;
        }
    }
}
