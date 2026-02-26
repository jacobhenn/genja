use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use cpal::{
    OutputCallbackInfo,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};

use crate::ParametricPath;

type RingbufProd<T> = <HeapRb<T> as ringbuf::traits::Split>::Prod;
type RingbufCons<T> = <HeapRb<T> as ringbuf::traits::Split>::Cons;

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, generate_audio);
    }
}

#[derive(Resource, Clone)]
struct AudioQueue {
    prod: Arc<Mutex<RingbufProd<f32>>>,
}

struct AudioStreamHandle {
    _stream: cpal::Stream,
}

#[derive(Resource)]
struct AudioDeviceConfig {
    sample_rate: u32,
    _channels: usize,
}

fn data_cb<T>(data: &mut [T], _info: &OutputCallbackInfo, cons: &mut RingbufCons<f32>)
where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    for sample_out in data {
        let sample = cons.try_pop().unwrap_or(0.0);
        *sample_out = T::from_sample(sample);
    }
}

pub fn setup_audio(app: &mut App) {
    let rb = Arc::new(HeapRb::<f32>::new(48000));
    let (prod, mut cons) = rb.split();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("host should have output device");
    let config = device.default_output_config().expect("device should have output config");

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();

    if channels != 2 {
        panic!("unsupported number of audio channels: {channels}");
    }

    let err_cb = |err| error!("audio error: {err}");

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data, _info| data_cb::<f32>(data, _info, &mut cons),
            err_cb,
            None,
        ),
        other => panic!("unsupported audio sample format {other}"),
    }
    .expect("should be able to build stream");

    stream.play().expect("stream should play");

    app.insert_non_send_resource(AudioStreamHandle { _stream: stream });
    app.insert_resource(AudioQueue { prod: Arc::new(Mutex::new(prod)) });
    app.insert_resource(AudioDeviceConfig { sample_rate, _channels: channels });
}

fn generate_audio(
    time: Res<Time>,
    audio: Res<AudioQueue>,
    config: Res<AudioDeviceConfig>,
    mut phase: Local<f32>,
    path: Res<ParametricPath>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    // dt = time since `generate_audio` was last called
    let dt = time.delta_secs();
    let n_frames = (dt * (config.sample_rate as f32)).ceil() as usize;

    // handle to an spsc ring buffer for audio samples consumed by the output device
    let mut prod = audio.prod.lock().unwrap();

    let (camera, camera_xform) = camera_query.single().expect("there is one camera");
    // let phys_vp_size = camera.physical_viewport_size().expect("viewport exists");

    let freq = 150.0;
    for _ in 0..n_frames {
        let pt_world = (path.f)(*phase);
        // ndc = normalized device coordinates; they live in [-1, 1] regardless of window size
        let pt_ndc = camera.world_to_ndc(camera_xform, pt_world).unwrap_or_else(|| {
            warn!("world_to_ndc failed");
            Vec3::new(0.0, 0.0, 0.0)
        });

        // a frame is 2 samples, left and right
        prod.try_push(pt_ndc.x.clamp_magnitude(1.0) * 0.5).expect("able to push to ringbuf");
        prod.try_push(pt_ndc.y.clamp_magnitude(1.0) * 0.5).expect("able to push to ringbuf");

        *phase += freq / config.sample_rate as f32;
        if *phase >= 1.0 {
            *phase -= 1.0;
        }
    }
}
