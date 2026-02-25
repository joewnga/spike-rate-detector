use rand::prelude::*;
use rand_distr::{Normal, Poisson, Distribution};
use plotters::prelude::*;
use std::error::Error;


pub fn detect_spike_rate(
    signal: &[f64],
    sampling_rate: f64,
    height_threshold: f64,
    min_distance_samples: usize,
) -> (f64, Vec<f64>) {
    if signal.len() < 3 {
        return (0.0, vec![]);
    }

    let mut peaks: Vec<usize> = Vec::new();
    let mut last_peak = 0usize;

    for i in 1..signal.len() - 1 {
        if signal[i] > signal[i - 1] && signal[i] > signal[i + 1] && signal[i] >= height_threshold {
            if i >= last_peak + min_distance_samples {
                peaks.push(i);
                last_peak = i;
            }
        }
    }

    let duration_sec = signal.len() as f64 / sampling_rate;
    let rate_hz = peaks.len() as f64 / duration_sec;
    let spike_times_sec: Vec<f64> = peaks.iter().map(|&idx| idx as f64 / sampling_rate).collect();

    (rate_hz, spike_times_sec)
}

fn main() -> Result<(), Box<dyn Error>> {
    
    let duration: f64 = 60.0;
    let sampling_rate: f64 = 1000.0;
    let num_samples: usize = (duration * sampling_rate) as usize;
    let expected_rate_hz: f64 = 5.0;

    let mut rng = thread_rng();   

    // Background noise
    let noise_dist = Normal::new(0.0, 0.2).unwrap();
    let mut signal: Vec<f64> = (0..num_samples)
        .map(|_| noise_dist.sample(&mut rng))
        .collect();

    // Poisson spikes
    let lambda = expected_rate_hz * duration;
    let num_spikes: usize = Poisson::new(lambda).unwrap().sample(&mut rng) as usize;

    // Random spike times
    let mut spike_indices: Vec<usize> = (0..num_spikes)
        .map(|_| (rng.gen_range(0.0..duration) * sampling_rate) as usize)
        .collect();
    spike_indices.sort_unstable();

    println!("Generated {} true spikes (expected ~{})", num_spikes, (expected_rate_hz * duration) as usize);

    // Add Gaussian spikes — explicit f64
    let spike_amplitude: f64 = 3.0;
    let spike_width: f64 = 5.0;

    for &idx in &spike_indices {
        let start = idx.saturating_sub(15);
        let end = (idx + 15).min(num_samples - 1);

        for i in start..=end {
            let x = (i as isize - idx as isize) as f64;
            let spike_value = spike_amplitude * (-x.powi(2) / (2.0 * spike_width.powi(2))).exp();
            signal[i] += spike_value;
        }
    }

    // Detect
    let (detected_rate, detected_times) = detect_spike_rate(&signal, sampling_rate, 1.0, 10);

    println!("Detected {} spikes", detected_times.len());
    println!("Spike rate = {:.2} Hz", detected_rate);

    // Plot
    let root = BitMapBackend::new("spike_rate.png", (1400, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Spike Detector (rand 0.8) — Rate = {:.2} Hz", detected_rate), ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..duration, -1.0..5.0)?;

    chart.configure_mesh().x_desc("Time (seconds)").y_desc("Voltage (a.u.)").draw()?;

    chart.draw_series(LineSeries::new(
        (0..num_samples).map(|i| (i as f64 / sampling_rate, signal[i])),
        &BLUE,
    ))?;

    chart.draw_series(PointSeries::of_element(
        detected_times.iter().map(|&t| (t, signal[(t * sampling_rate) as usize])),
        8,
        ShapeStyle::from(&RED).stroke_width(2),
        &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
    ))?;

    println!("\nPlot saved as spike_rate.png — open it now!");
    Ok(())
}