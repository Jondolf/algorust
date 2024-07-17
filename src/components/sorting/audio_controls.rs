use std::collections::HashMap;

use crate::components::input_items::*;

use instant::Duration;
use web_sys::OscillatorType;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AudioConfig {
    pub enabled: bool,
    pub sound_type: OscillatorType,
    pub min_frequency: f32,
    pub max_frequency: f32,
    pub note_duration: Duration,
}
impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_type: OscillatorType::Sine,
            min_frequency: 55.0, // A1 in standard tuning
            max_frequency: 880.0, // A5 in standard tuning
            note_duration: Duration::from_millis(200),
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct AudioControlsProps {
    pub config: AudioConfig,
    pub update_config: Callback<(AudioConfig, bool)>,
}

#[function_component]
pub fn AudioControls(props: &AudioControlsProps) -> Html {
    let AudioControlsProps {
        config,
        update_config,
    } = props.clone();
    let sound_types: UseStateHandle<HashMap<&str, OscillatorType>> = use_state_eq(|| {
        HashMap::from([
            ("Sawtooth", OscillatorType::Sawtooth),
            ("Sine", OscillatorType::Sine),
            ("Square", OscillatorType::Square),
            ("Triangle", OscillatorType::Triangle),
        ])
    });

    let toggle_audio = {
        let config = config.clone();
        let update_config = update_config.clone();

        Callback::from(move |_| {
            update_config.emit((
                AudioConfig {
                    enabled: !config.enabled,
                    ..config.clone()
                },
                false,
            ));
        })
    };

    let change_sound_type = {
        let config = config.clone();
        let update_config = update_config.clone();
        let sound_types = sound_types.clone();

        Callback::from(move |sound_type: String| {
            update_config.emit((
                AudioConfig {
                    sound_type: sound_types.get(sound_type.as_str()).unwrap().to_owned(),
                    ..config.clone()
                },
                false,
            ))
        })
    };

    let change_min_frequency = {
        let config = config.clone();
        let update_config = update_config.clone();

        Callback::from(move |min_frequency| {
            update_config.emit((
                AudioConfig {
                    min_frequency,
                    ..config.clone()
                },
                false,
            ))
        })
    };

    let change_max_frequency = {
        let config = config.clone();

        Callback::from(move |max_frequency| {
            update_config.emit((
                AudioConfig {
                    max_frequency,
                    ..config.clone()
                },
                false,
            ))
        })
    };

    html! {
        <div class="audio-controls">
            <Checkbox title="Audio enabled" value={config.enabled} oninput={toggle_audio} />
            <SelectInput
                title="Sound type"
                options={sound_types.keys().into_iter().map(|key| key.to_string()).collect::<Vec<String>>()}
                selected_value={sound_types.clone().iter().find(|(_, val)| **val == config.sound_type).unwrap().0.to_owned()}
                onchange={change_sound_type}
            />
            <FloatInput<f32>
                title="Minimum frequency"
                value={props.config.min_frequency}
                oninput={change_min_frequency}
                min={0.0}
                step={10.0}
            />
            <FloatInput<f32>
                title="Maximum frequency"
                value={props.config.max_frequency}
                oninput={change_max_frequency}
                min={0.0}
                step={10.0}
            />
        </div>
    }
}
