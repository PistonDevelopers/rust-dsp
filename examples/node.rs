//! 
//! An example of using dsp-chain's `Node` trait to create a simple
//! Synthesiser with 3 sine wave oscillators.
//!

extern crate dsp;
extern crate num;

use dsp::{Event, Node, Sample, Settings, SoundStream, Wave};

/// SoundStream is currently generic over i8, i32 and f32. Feel free to change it!
type AudioSample = f32;

type Input = AudioSample;
type Output = AudioSample;

type Phase = f64;
type Frequency = f64;
type Volume = f32;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

fn main() {

    // Construct the stream and handle any errors that may have occurred.
    let mut stream = match SoundStream::<Input, Output>::new().run() {
        Ok(stream) => { println!("It begins!"); stream },
        Err(err) => panic!("An error occurred while constructing SoundStream: {}", err),
    };

    // Construct our fancy Synth!
    let mut synth = Synth([
            Oscillator(0.0, A5_HZ, 0.2), 
            Oscillator(0.0, D5_HZ, 0.1), 
            Oscillator(0.0, F5_HZ, 0.15)
        ]);

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // The SoundStream iterator will automatically return these events in this order.
    for event in stream.by_ref() {
        match event {
            Event::Out(buffer, settings) => synth.audio_requested(buffer, settings),
            Event::Update(dt) => if timer > 0.0 { timer -= dt } else { break },
            _ => (),
        }
    }

    // Close the stream and shut down PortAudio.
    match stream.close() {
        Ok(()) => println!("Great success!"),
        Err(err) => println!("An error occurred while closing SoundStream: {}", err),
    }

}


/// Synth will be our demonstration of a parent DspNode where the Oscillators
/// that it owns are it's children.
struct Synth([Oscillator; 3]);

impl Node<Output> for Synth {
    /// Here we return a reference to each of our Oscillators as our `inputs`.
    /// This allows the default `audio_requested` method to draw input from
    /// each of our oscillators automatically.
    fn inputs(&mut self) -> Vec<&mut Node<Output>> {
        let Synth(ref mut oscillators) = *self;
        oscillators.iter_mut().map(|osc| osc as &mut Node<Output>).collect()
    }
}


/// Oscillator will be our generator type of node, meaning that we will override
/// the way it provides audio via its `audio_requested` method.
struct Oscillator(Phase, Frequency, Volume);

impl Node<Output> for Oscillator {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut [Output], settings: Settings) {
        let Oscillator(ref mut phase, frequency, volume) = *self;
        for frame in buffer.chunks_mut(settings.channels as usize) {
            *phase += frequency / settings.sample_hz as f64;
            let val = sine_wave(*phase, volume);
            for channel in frame.iter_mut() {
                *channel = val;
            }
        }
    }
}

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S {
    use std::f64::consts::PI;
    use num::Float;
    Sample::from_wave((phase * PI * 2.0).sin() as Wave * volume)
}

