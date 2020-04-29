//! Tools for measuring metrics statistics.
//!
//! # Usage
//!
//! TODO: Examples
//!
//! # Implementation
//!
//! TODO: Explain implementation
use crate::number::NonNan;
use chrono::prelude::*;
use crossbeam::{bounded, select, tick, unbounded, Receiver, Sender};
use log::{trace, warn};
use std::cmp::Ord;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub mod builtin_metrics {
    //! Type ids for metrics internal to the engine.

    /// Time taken to update scene.
    pub const SCENE_UPDATE: u16 = 1000;
    /// Time taken sending the graphics encoder to gfx.
    pub const GRAPHICS_RENDER: u16 = 2000;
}

/// Central hub for recording and aggregating metrics.
pub struct MetricHub {
    timeseries_map: Arc<Mutex<BTreeMap<MetricKey, TimeSeries>>>,
    worker_handle: Option<thread::JoinHandle<()>>,
    message_sender: Sender<MetricMessage>,
    cancel_send: Sender<()>,
    settings: MetricSettings,
}

impl Default for MetricHub {
    fn default() -> Self {
        MetricHub::new(MetricSettings {
            data_point_count: 64,
            aggregate_interval: Duration::from_secs(1),
        })
    }
}

impl MetricHub {
    pub fn new(settings: MetricSettings) -> MetricHub {
        let (message_sender, message_recv) = unbounded::<MetricMessage>();
        let (cancel_send, cancel_recv) = bounded::<()>(1);

        let timeseries_map = Arc::new(Mutex::new(BTreeMap::new()));

        let worker_handle = MetricHub::spawn_thread(
            settings.clone(),
            Arc::clone(&timeseries_map),
            message_recv,
            cancel_recv,
        );

        MetricHub {
            timeseries_map,
            worker_handle: Some(worker_handle),
            message_sender,
            cancel_send,
            settings,
        }
    }

    fn spawn_thread(
        settings: MetricSettings,
        timeseries_map: Arc<Mutex<BTreeMap<MetricKey, TimeSeries>>>,
        message_recv: Receiver<MetricMessage>,
        cancel_recv: Receiver<()>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let ticker = tick(settings.aggregate_interval);

            'message_pump: loop {
                select! {
                    recv(message_recv) -> maybe_msg => {
                        if let Ok(msg) = maybe_msg {
                            let mut ts_map = timeseries_map
                                .lock()
                                .expect("Metric worker mutex poisoned");
                            let timeseries = ts_map
                                .entry(msg.key())
                                .or_insert_with(|| {
                                    TimeSeries::new(settings.aggregate_interval, settings.data_point_count)
                                });
                            // Convert metrics into raw measurements.
                            timeseries
                                .measurements
                                .entry(msg.slot(timeseries.interval)
                                          .expect("divide by zero"))
                                .or_insert_with(|| vec![])
                                .push(msg.into());
                        }
                    }
                    recv(ticker) -> _instant => {
                        let mut ts_map = timeseries_map
                            .lock()
                            .expect("Metric worker mutex poisoned");
                        for (key, timeseries) in ts_map.iter_mut() {
                            process_timeseries(key.aggregate, timeseries);
                        }
                    }
                    recv(cancel_recv) -> _msg => {
                        break 'message_pump;
                    }
                }
                // So we don't starve other threads.
                thread::yield_now();
            }
            trace!("Metric worker thread shut down.");
        })
    }

    /// Measure time taken by a block of code.
    pub fn timer(&self, metric_id: u16, aggregate: MetricAggregate) -> TimerMetric {
        TimerMetric {
            sender: self.message_sender.clone(),
            metric_id,
            start_at: Instant::now(),
            aggregate,
            stopped: false,
        }
    }

    /// Builds a time series, containing aggregated datapoints.
    pub fn make_time_series(
        &self,
        metric_id: u16,
        aggregate: MetricAggregate,
        out: &mut [DataPoint],
        start: usize,
        length: usize,
    ) {
        let mut timeseries_map = self
            .timeseries_map
            .lock()
            .expect("Metric hub mutex has been poisoned");
        let timeseries = timeseries_map
            .entry(MetricKey {
                metric_id,
                aggregate,
            })
            .or_insert_with(|| {
                TimeSeries::new(
                    self.settings.aggregate_interval,
                    self.settings.data_point_count,
                )
            });
        let mut index = start;
        for data_point in timeseries.data_points.iter().take(length) {
            out[index] = data_point.clone();
            index += 1;
        }
    }
}

impl Drop for MetricHub {
    fn drop(&mut self) {
        trace!("Dropping metric hub. Waiting for worker thread to shut down.");
        if let Err(err) = self.cancel_send.send(()) {
            warn!("Cancel metric worker send error: {}", err);
        }
        if let Some(join_handle) = self.worker_handle.take() {
            join_handle
                .join()
                .expect("Couldn't join metric worker thread.");
        }
    }
}

fn process_timeseries(aggregate: MetricAggregate, timeseries: &mut TimeSeries) {
    if let Some(slot) = timeseries.measurements.iter().map(|(key, _)| *key).next() {
        // Important: remove element to cleanup memory.
        if let Some(measurements) = timeseries.measurements.remove(&slot) {
            match aggregate {
                MetricAggregate::Maximum => {
                    let max_value = measurements
                        .into_iter()
                        .map(|raw| NonNan::new(raw.value).expect("Metric value was nan"))
                        .max();
                    let naive = NaiveDateTime::from_timestamp(slot, 0);
                    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                    timeseries.data_points.push_back(DataPoint {
                        datetime: datetime.into(),
                        value: max_value.unwrap().into_inner(),
                    });
                }
                _ => warn!("Aggregate {:?} unimplemented", aggregate),
            }
        }
    }

    // Important: Limit the size of the time series for memory usage.
    if timeseries.data_points.len() > timeseries.max_data_points {
        let overflow = timeseries.data_points.len() - timeseries.max_data_points;
        for _ in 0..overflow {
            timeseries.data_points.pop_front();
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricSettings {
    /// Number of data points to keep in history.
    data_point_count: usize,
    /// Interval on which the background worker thread aggregates measurements
    /// into data points.
    aggregate_interval: Duration,
}

impl Default for MetricSettings {
    fn default() -> Self {
        MetricSettings {
            data_point_count: 64,
            aggregate_interval: Duration::from_secs(1),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MetricAggregate {
    Minimum,
    Maximum,
    Average,
    Sum,
    Count,
    P95,
    P99,
}

pub struct TimerMetric {
    sender: Sender<MetricMessage>,
    metric_id: u16,
    start_at: Instant,
    aggregate: MetricAggregate,
    stopped: bool,
}

impl TimerMetric {
    #[inline]
    pub fn stopped(&self) -> bool {
        self.stopped
    }

    pub fn stop(&mut self) {
        if !self.stopped {
            // println!("Stop timer");
            let msg = MetricMessage::TimeMeasurement {
                key: MetricKey {
                    metric_id: self.metric_id,
                    aggregate: self.aggregate,
                },
                duration: self.start_at.elapsed(),
                datetime: Local::now(),
            };

            if let Err(err) = self.sender.send(msg) {
                warn!("Timer failed to record metric: {}", err);
            }
            self.stopped = true;
        }
    }
}

impl Drop for TimerMetric {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct MetricKey {
    metric_id: u16,
    aggregate: MetricAggregate,
}

/// Message for the metric hub to communicate with the worker thread.
#[allow(dead_code)]
enum MetricMessage {
    TimeMeasurement {
        key: MetricKey,
        duration: Duration,
        datetime: DateTime<Local>,
    },
}

impl MetricMessage {
    #[inline]
    fn key(&self) -> MetricKey {
        match self {
            MetricMessage::TimeMeasurement { key, .. } => *key,
        }
    }

    /// Calculate the slot this message belongs to.
    #[inline]
    fn slot(&self, interval: Duration) -> Option<i64> {
        let datetime = match self {
            MetricMessage::TimeMeasurement { datetime, .. } => *datetime,
        };

        let timestamp = datetime.timestamp_millis();
        let interval_millis = interval.as_millis() as i64;

        if interval_millis == 0 {
            // Divide by zero.
            None
        } else {
            // Integer division rounds down.
            Some(timestamp / interval_millis)
        }
    }
}

/// Aggregated metrics.
struct TimeSeries {
    interval: Duration,
    measurements: BTreeMap<i64, Vec<RawMeasurement>>,
    data_points: VecDeque<DataPoint>,
    max_data_points: usize,
}

impl TimeSeries {
    fn new(interval: Duration, max_data_points: usize) -> Self {
        TimeSeries {
            interval,
            measurements: BTreeMap::new(),
            data_points: VecDeque::new(),
            max_data_points,
        }
    }
}

#[derive(Debug, Clone)]
struct RawMeasurement {
    timestamp: i64,
    value: f64,
}

impl From<MetricMessage> for RawMeasurement {
    fn from(m: MetricMessage) -> Self {
        match m {
            MetricMessage::TimeMeasurement {
                duration, datetime, ..
            } => RawMeasurement {
                // Duration as float milliseconds
                value: (duration.as_nanos() as f64) / 1_000_000.0,
                timestamp: datetime.timestamp(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub datetime: DateTime<Local>,
    pub value: f64,
}

impl Default for DataPoint {
    #[inline]
    fn default() -> Self {
        DataPoint {
            datetime: Local::now(),
            value: 0.,
        }
    }
}
