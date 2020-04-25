use chrono::prelude::*;
use crossbeam::{bounded, select, tick, unbounded, Receiver, Sender};
use log::{trace, warn};
use std::borrow::Borrow;
use std::collections::btree_map;
use std::collections::{BTreeMap, VecDeque};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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
                                    TimeSeries {
                                        interval: Duration::from_secs(1),
                                        measurements: BTreeMap::new(),
                                        data_points: VecDeque::new()
                                    }
                                });

                            timeseries
                                .measurements
                                .entry(msg.slot(timeseries.interval)
                                          .expect("divide by zero"))
                                .or_insert_with(|| vec![])
                                .push(msg);
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
                    recv(cancel_recv) -> _msg => break 'message_pump,
                }
            }
            trace!("Metric worker thread shut down.");
        })
    }

    pub fn timer(&self, metric_id: u16, aggregate: MetricAggregate) -> TimerMetric<'_> {
        TimerMetric {
            hub: self,
            metric_id,
            start_at: Instant::now(),
            aggregate,
        }
    }

    pub fn time_series(&self, metric_id: u16, aggregate: MetricAggregate) -> TimeSeriesRef<'_> {
        // let mut ts_map = self.timeseries.lock().expect("Metric hub mutex poisoned");
        // let key = MetricKey {
        //     metric_id,
        //     aggregate,
        // };

        // TimeSeriesRef {
        //     time_series: ts_map.entry(key).or_insert_with(|| TimeSeries {
        //         data_points: VecDeque::new(),
        //     }),
        // }
        unimplemented!()
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

fn process_timeseries(aggregate: MetricAggregate, timeseries: &mut TimeSeries) {}

#[derive(Debug, Clone)]
pub struct MetricSettings {
    /// Number of data points to keep in history.
    data_point_count: usize,
    /// Interval on which the background worker thread aggregates measurements
    /// into data points.
    aggregate_interval: Duration,
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

pub struct TimerMetric<'a> {
    hub: &'a MetricHub,
    metric_id: u16,
    start_at: Instant,
    aggregate: MetricAggregate,
}

impl<'a> Drop for TimerMetric<'a> {
    fn drop(&mut self) {
        // TODO: record to hub
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct MetricKey {
    metric_id: u16,
    aggregate: MetricAggregate,
}

/// Message for the metric hub to communicate with the worker thread.
enum MetricMessage {
    TimeMeasurement {
        key: MetricKey,
        duration: Duration,
        datetime: DateTime<Local>,
    },
    IncrMeasurement {
        key: MetricKey,
        amount: u32,
        datetime: DateTime<Local>,
    },
}

impl MetricMessage {
    #[inline]
    fn key(&self) -> MetricKey {
        match self {
            MetricMessage::TimeMeasurement { key, .. } => *key,
            MetricMessage::IncrMeasurement { key, .. } => *key,
        }
    }

    /// Calculate the slot this message belongs to.
    #[inline]
    fn slot(&self, interval: Duration) -> Option<i64> {
        let datetime = match self {
            MetricMessage::TimeMeasurement { datetime, .. } => *datetime,
            MetricMessage::IncrMeasurement { datetime, .. } => *datetime,
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
pub struct TimeSeries {
    interval: Duration,
    measurements: BTreeMap<i64, Vec<MetricMessage>>,
    data_points: VecDeque<f32>,
}

pub struct DataPoint {
    timestamp: u32,
    value: f32,
}

/// A borrowed reference to a time series.
pub struct TimeSeriesRef<'a> {
    time_series: &'a TimeSeries,
}

impl<'a> Deref for TimeSeriesRef<'a> {
    type Target = TimeSeries;

    fn deref(&self) -> &Self::Target {
        self.time_series
    }
}

impl<'a> Borrow<TimeSeries> for TimeSeriesRef<'a> {
    fn borrow(&self) -> &TimeSeries {
        self.time_series
    }
}
