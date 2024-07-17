use crate::{
    components::{
        algo_desc::AlgoDesc,
        collapsible::Collapsible,
        sidebar::Sidebar,
        sorting::{
            audio_controls::{AudioConfig, AudioControls},
            sort_controls::SortControls,
            sort_graph::SortGraph,
        },
        step_slider::StepSlider,
    },
    hooks::use_sort_audio::use_sort_audio,
    utils::{gen_u32_vec, knuth_shuffle},
};
use sorting::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use yew::prelude::*;
use yew_hooks::use_title;
use yew_router::prelude::*;

type SortSteps = Vec<Vec<SortCommand<u32>>>;

#[derive(Clone)]
pub struct SortingAlgorithm {
    pub name: String,
    sort: fn(&mut Vec<u32>, &mut SortSteps),
}

impl SortingAlgorithm {
    fn new(name: &str, sort: fn(&mut Vec<u32>, &mut SortSteps)) -> Self {
        Self {
            name: name.to_string(),
            sort,
        }
    }
    fn sort(&self, input: Rc<RefCell<Vec<u32>>>) -> SortResult<u32> {
        run_sort(input, self.sort)
    }
}

impl Default for SortingAlgorithm {
    fn default() -> Self {
        Self {
            name: String::from("Bubble sort"),
            sort: bubble_sort,
        }
    }
}

// `SortingAlgorithm` has to implement `PartialEq` because of `Properties` requirements, but it can't be derived because of the `sort` function that takes a mutable reference.
// Here we implement it manually, assuming that two `SortingAlgorithm`s are partially equal if their names are the same.
impl PartialEq for SortingAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub fn get_sorting_algorithms() -> BTreeMap<&'static str, SortingAlgorithm> {
    // `BTreeMap` because it keeps the order of the items.
    BTreeMap::from([
        (
            "bubble-sort",
            SortingAlgorithm::new("Bubble sort", bubble_sort),
        ),
        (
            "insertion-sort",
            SortingAlgorithm::new("Insertion sort", insertion_sort),
        ),
        (
            "merge-sort",
            SortingAlgorithm::new("Merge sort", merge_sort),
        ),
        ("heapsort", SortingAlgorithm::new("Heapsort", heapsort)),
        ("quicksort", SortingAlgorithm::new("Quicksort", quicksort)),
        (
            "bucket-sort",
            SortingAlgorithm::new("Bucket sort", bucket_sort),
        ),
    ])
}

#[derive(Clone, Debug, Routable, PartialEq, Eq)]
pub enum SortingRoute {
    #[at("/sorting")]
    Sorting,
    #[at("/sorting/:algorithm")]
    SortingAlgorithm { algorithm: String },
}

pub fn switch_sorting(route: SortingRoute) -> Html {
    match route {
        SortingRoute::Sorting => html! {
            <Redirect<SortingRoute> to={SortingRoute::SortingAlgorithm { algorithm: "bubble-sort".to_string()} } />
        },
        SortingRoute::SortingAlgorithm { algorithm } => {
            if get_sorting_algorithms().contains_key(algorithm.as_str()) {
                html! {
                    <SortingAlgorithmsPage algorithm={algorithm} />
                }
            } else {
                html! {
                    <Sorting404Page algorithm={algorithm} />
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SortConfig {
    pub input_len: usize,
    pub sorting_algorithm: SortingAlgorithm,
    pub audio_enabled: bool,
    /// How long the playback of steps should take in seconds.
    pub playback_time: f32,
    pub audio_config: AudioConfig,
}
impl Default for SortConfig {
    fn default() -> Self {
        Self {
            input_len: 96, // LCM of 12 & 32; useful for pitch and computation
            sorting_algorithm: SortingAlgorithm::default(),
            audio_enabled: true,
            playback_time: 10.0,
            audio_config: AudioConfig::default(),
        }
    }
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct SortingAlgorithmsPageProps {
    #[prop_or("bubble-sort".to_string())]
    pub algorithm: String,
}

#[function_component]
pub fn SortingAlgorithmsPage(props: &SortingAlgorithmsPageProps) -> Html {
    let config = {
        let algorithm_name = props.algorithm.to_string();

        use_state(|| {
            let mut config = SortConfig::default();
            if let Some(algorithm) = get_sorting_algorithms().get(algorithm_name.as_str()) {
                config.sorting_algorithm = algorithm.to_owned();
            }
            config
        })
    };

    let input = use_mut_ref(|| knuth_shuffle(gen_u32_vec(config.input_len)));
    let output = use_mut_ref(|| input.borrow().clone());
    let sort_result = use_mut_ref(|| config.sorting_algorithm.sort(Rc::clone(&output)));

    // The active step is empty at the input step, step 0.
    let active_step = use_state(std::vec::Vec::<SortCommand<u32>>::new);
    let active_step_index: UseStateHandle<usize> = use_state_eq(|| 0);

    {
        let active_step = active_step.clone();
        let sort_result = Rc::clone(&sort_result);

        use_effect_with_deps(
            move |i| {
                active_step.set(if **i == 0 {
                    vec![]
                } else {
                    sort_result.borrow().steps[**i - 1].clone()
                });

                || ()
            },
            active_step_index.clone(),
        );
    }

    let output_at_active_step = use_state(|| {
        get_output_at_step_index(
            &input.borrow(),
            &sort_result.borrow().steps,
            *active_step_index,
        )
    });

    let route = use_route::<SortingRoute>();

    let update_values = {
        let input = input.clone();
        let sort_result = sort_result.clone();
        let active_step_index = active_step_index.clone();
        let output_at_active_step = output_at_active_step.clone();

        move |new_input: Vec<u32>, config: &SortConfig| {
            // Cloning here is necessary if we want to keep the input
            *input.borrow_mut() = new_input.clone();
            *output.borrow_mut() = new_input;
            *sort_result.borrow_mut() = config.sorting_algorithm.sort(Rc::clone(&output));
            active_step_index.set(0);

            output_at_active_step.set(get_output_at_step_index(
                &input.borrow(),
                &sort_result.borrow().steps,
                0,
            ));
        }
    };

    let update_input = {
        let config = config.clone();
        let update_values = update_values.clone();

        Callback::from(move |val| {
            update_values(val, &config);
        })
    };

    let update_config = {
        let config = config.clone();
        let update_values = update_values.clone();

        Callback::from(move |msg: (SortConfig, bool)| {
            if msg.1 {
                let new_input = knuth_shuffle(gen_u32_vec(msg.0.input_len));
                update_values(new_input, &msg.0);
            }
            config.set(msg.0);
        })
    };

    let update_audio_config = {
        let config = config.clone();
        let update_values = update_values.clone();

        Callback::from(move |msg: (AudioConfig, bool)| {
            if msg.1 {
                let new_input = knuth_shuffle(gen_u32_vec(config.input_len));
                update_values(new_input, &config);
            }
            config.set(SortConfig {
                audio_config: msg.0,
                ..(*config).clone()
            });
        })
    };

    let change_step = {
        let active_step_index = active_step_index.clone();
        let sort_result = Rc::clone(&sort_result);
        let output_at_active_step = output_at_active_step.clone();

        Callback::from(move |val: usize| {
            output_at_active_step.set(get_output_at_step_index(
                &input.borrow(),
                &sort_result.borrow().steps,
                val,
            ));
            active_step_index.set(val);
        })
    };

    {
        let config = config.clone();
        let update_config = update_config.clone();

        use_effect_with_deps(
            move |route| {
                let algorithm_name = match route.as_ref().unwrap() {
                    SortingRoute::SortingAlgorithm { algorithm } => algorithm,
                    _ => "bubble-sort",
                };
                if let Some(algorithm) = get_sorting_algorithms().get(algorithm_name) {
                    update_config.emit((
                        SortConfig {
                            sorting_algorithm: algorithm.to_owned(),
                            ..(*config).clone()
                        },
                        true,
                    ))
                } else {
                    update_config.emit(((*config).clone(), false));
                };
                || ()
            },
            route,
        );
    }

    use_title(format!(
        "{} - Sorting algorithms",
        config.sorting_algorithm.name
    ));

    use_sort_audio(
        output_at_active_step.clone(),
        active_step.clone(),
        config.audio_config.clone(),
    );

    html! {
        <div class="page" id="SortingAlgorithms">
            <Sidebar>
                <h2>{"Config"}</h2>

                <Collapsible title="General" open={true} class="config-section">
                    <SortControls config={(*config).clone()} {update_input} {update_config} />
                </Collapsible>

                <Collapsible title="Audio" open={false} class="config-section">
                    <AudioControls config={config.audio_config.clone()} update_config={update_audio_config} />
                </Collapsible>
            </Sidebar>

            <main>
                <div class="visualization">
                    <span>{ format!("{} steps, {:?} ms", sort_result.borrow().steps.len(), &sort_result.borrow().duration.unwrap().as_millis()) }</span>

                    <SortGraph
                        items={output_at_active_step.clone()}
                        step={active_step.clone()}
                    />

                    <StepSlider
                        active_step_index={*active_step_index}
                        max={sort_result.borrow().steps.len()}
                        on_change={change_step}
                        playback_time={config.playback_time}
                    />

                    <span class="step-info">
                        <label for="stepSlider">
                            {
                                if *active_step_index == 0 {
                                    format!("Step {} (input)", *active_step_index)
                                } else {
                                    format!("Step {}: ", *active_step_index)
                                }
                            }
                        </label>
                        {
                            active_step.iter().map(|command| match command {
                                SortCommand::Swap(from, to) => format!("SWAP indices {} and {}", from, to),
                                SortCommand::Set(i, val) => format!("SET value at index {} to {}", i, val),
                            }).collect::<Vec<String>>().join(";")
                        }
                    </span>
                </div>

                <AlgoDesc algorithm={config.sorting_algorithm.name.clone()} />
            </main>
        </div>
    }
}

/// Gets the output at a given step's index by running [`SortCommand`]s on the input.
fn get_output_at_step_index(
    input: &[u32],
    steps: &[Vec<SortCommand<u32>>],
    index: usize,
) -> Vec<u32> {
    let mut output = input.to_vec();
    run_sort_steps(&mut output, &steps[0..index]);
    output
}

#[derive(Clone, PartialEq, Properties)]
struct Sorting404PageProps {
    algorithm: String,
}

#[function_component]
fn Sorting404Page(props: &Sorting404PageProps) -> Html {
    use_title("404 - Sorting".to_string());

    html! {
        <>
            <h1>{ "404" }</h1>
            <p>{ format!("The algorithm \"{}\" was not found.", props.algorithm) }</p>
            <Link<SortingRoute> to={SortingRoute::Sorting}>
                { "Back to sorting" }
            </Link<SortingRoute>>
        </>
    }
}
