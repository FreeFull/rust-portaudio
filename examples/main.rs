#![feature(globs)]
#![allow(unreachable_code, unused_assignments)]

extern crate portaudio;

use portaudio::*;
use std::error::Error;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;

fn main() -> () {

    println!("PortAudio version : {}", pa::get_version());
    println!("PortAudio version text : {}", pa::get_version_text());

    match pa::initialize() {
        Ok(()) => println!("Successfully initialized PortAudio"),
        Err(err) => println!("An error occurred while initializing PortAudio: {}", err.description()),
    }

    println!("PortAudio host count : {}", pa::host::get_api_count() as int);

    let default_host = pa::host::get_default_api();
    println!("PortAudio default host : {}", default_host as int);

    match pa::host::get_api_info(default_host) {
        None => println!("Couldn't retrieve api info for the default host."),
        Some(info) => println!("PortAudio host name : {}", info.name),
    }

    let type_id = pa::host::api_type_id_to_host_api_index(pa::HostApiTypeId::CoreAudio) as int;
    println!("PortAudio type id : {}", type_id);

    let def_input = pa::device::get_default_input();
    let input_info = match pa::device::get_info(def_input) {
        Ok(info) => info,
        Err(err) => panic!("An error occurred while retrieving input info: {}", err.description()),
    };
    println!("Default input device info :");
    println!("\tversion : {}", input_info.struct_version);
    println!("\tname : {}", input_info.name);
    println!("\tmax input channels : {}", input_info.max_input_channels);
    println!("\tmax output channels : {}", input_info.max_output_channels);
    println!("\tdefault sample rate : {}", input_info.default_sample_rate);

    // Construct the input stream parameters.
    let input_stream_params = pa::StreamParameters {
        device : def_input,
        channel_count : 2,
        sample_format : pa::SampleFormat::Float32,
        suggested_latency : input_info.default_low_input_latency
    };

    let def_output = pa::device::get_default_output();
    let output_info = match pa::device::get_info(def_output) {
        Ok(info) => info,
        Err(err) => panic!("An error occurred while retrieving output info: {}", err.description()),
    };

    println!("Default output device name : {}", output_info.name);

    // Construct the output stream parameters.
    let output_stream_params = pa::StreamParameters {
        device : def_output,
        channel_count : 2,
        sample_format : pa::SampleFormat::Float32,
        suggested_latency : output_info.default_low_output_latency
    };

    let mut stream : pa::Stream<f32, f32> = pa::Stream::new();

    match stream.open(Some(&input_stream_params),
                      Some(&output_stream_params),
                      SAMPLE_RATE,
                      FRAMES,
                      pa::StreamFlags::ClipOff) {
        Ok(()) => println!("Successfully opened the stream."),
        Err(err) => println!("An error occurred while opening the stream: {}", err.description()),
    }

    match stream.start() {
        Ok(()) => println!("Successfully started the stream."),
        Err(err) => println!("An error occurred while starting the stream: {}", err.description()),
    }

    // We'll use this function to wait for read/write availability.
    let wait_for_stream = |f: || -> Result<Option<i64>, pa::error::Error>, name: &str| {
        'waiting_for_stream: loop {
            match f() {
                Ok(None) => (),
                Ok(Some(frames)) => {
                    println!("{} stream available with {} frames.", name, frames);
                    break 'waiting_for_stream
                },
                Err(err) => panic!("An error occurred while waiting for the {} stream: {}", name, err.description()),
            }
        }
    };

    // Now start the main read/write loop! In this example, we pass the input buffer directly to
    // the output buffer, so watch out for feedback.
    'stream: loop {
        wait_for_stream(|| stream.get_stream_read_available(), "Read");
        match stream.read(FRAMES) {
            Ok(input_samples)  => {
                wait_for_stream(|| stream.get_stream_write_available(), "Write");
                match stream.write(input_samples, FRAMES) {
                    Ok(()) => (),
                    Err(err) => {
                        println!("An error occurred while writing to the output stream: {}", err.description());
                        break 'stream
                    },
                }
            },
            Err(err) => {
                println!("An error occurred while reading from the input stream: {}", err.description());
                break 'stream
            }
        };
    }

    match stream.close() {
        Ok(()) => println!("Successfully closed the stream."),
        Err(err) => println!("An error occurred while closing the stream: {}", err.description()),
    }

    println!("");

    match pa::terminate() {
        Ok(()) => println!("Successfully terminated PortAudio."),
        Err(err) => println!("An error occurred while terminating PortAudio: {}", err.description()),
    }

}
