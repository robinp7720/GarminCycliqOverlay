use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use fitparser;
use fitparser::profile::MesgNum;
use fitparser::{FitDataRecord, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::ops::Add;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RecordDataField {
    pub timestamp: NaiveDateTime,
    pub lat: Option<i32>,
    pub long: Option<i32>,
    pub hr: Option<u8>,
    pub cadence: Option<u8>,
    pub distance: f64,
    pub power: Option<u16>,
    pub temperature: Option<i8>,
    pub accumulated_power: Option<u32>,
    pub fractional_cadence: Option<f64>,
    pub enhanced_speed: f64,
    pub enhanced_altitude: Option<f64>,
}

pub fn parse_fit_data_record(
    record: FitDataRecord,
    start_time: Option<chrono::DateTime<Local>>,
) -> RecordDataField {
    let mut timestamp: Option<NaiveDateTime> = None;
    let mut lat: Option<i32> = None;
    let mut long: Option<i32> = None;
    let mut hr: Option<u8> = None;
    let mut cadence: Option<u8> = None;
    let mut distance: f64 = 0.0;
    let mut power: Option<u16> = None;
    let mut temperature: Option<i8> = None;
    let mut accumulated_power: Option<u32> = None;
    let mut fractional_cadence: Option<f64> = None;
    let mut enhanced_speed: f64 = 0.0;
    let mut enhanced_altitude: Option<f64> = None;

    for field in record.fields() {
        match field.name() {
            "position_lat" => {
                lat = match field.value() {
                    Value::SInt32(v) => Some(*v),
                    _ => None,
                };
            }
            "position_long" => {
                long = match field.value() {
                    Value::SInt32(v) => Some(*v),
                    _ => None,
                };
            }
            "heart_rate" => {
                hr = match field.value() {
                    Value::UInt8(v) => Some(*v),
                    _ => None,
                };
            }
            "cadence" => {
                cadence = match field.value() {
                    Value::UInt8(v) => Some(*v),
                    _ => None,
                };
            }
            "distance" => {
                distance = match field.value() {
                    Value::Float64(v) => *v,
                    _ => 0.0,
                };
            }
            "power" => {
                power = match field.value() {
                    Value::UInt16(v) => Some(*v),
                    _ => None,
                };
            }
            "temperature" => {
                temperature = match field.value() {
                    Value::SInt8(v) => Some(*v),
                    _ => None,
                };
            }
            "accumulated_power" => {
                accumulated_power = match field.value() {
                    Value::UInt32(v) => Some(*v),
                    _ => None,
                };
            }
            "fractional_cadence" => {
                fractional_cadence = match field.value() {
                    Value::Float64(v) => Some(*v),
                    _ => None,
                };
            }
            "enhanced_speed" => {
                enhanced_speed = match field.value() {
                    Value::Float64(v) => *v,
                    _ => 0.0,
                };
            }
            "timestamp" => {
                timestamp = match field.value() {
                    Value::Timestamp(v) => Some(v.naive_local()),
                    _ => None,
                };
            }
            _ => {
                println!("{:#?}", field.name());
            }
        }
    }

    return RecordDataField {
        timestamp: timestamp.expect("Timestamp not found"),
        lat,
        long,
        hr,
        cadence,
        distance,
        power,
        temperature,
        accumulated_power,
        fractional_cadence,
        enhanced_speed,
        enhanced_altitude,
    };
}

pub fn parse_fit_file(file_path: &str) -> Vec<RecordDataField> {
    let mut fp = File::open(file_path).unwrap();
    let data = fitparser::from_reader(&mut fp).unwrap();

    let mut linearization: Vec<RecordDataField> = vec![];

    // Find the first Record message to get the start time
    //let mut start_time: Option<DateTime<Local>> = None;

    let start_time: Option<DateTime<Local>> = Some(
        DateTime::parse_from_rfc3339("2023-05-25T17:15:45+02:00")
            .unwrap()
            .into(),
    );

    /*for entry in &data {
        match entry.kind() {
            MesgNum::Record => {
                let parsed = parse_fit_data_record(entry.clone(), start_time);
                start_time = Some(parsed.timestamp);
                break;
            }
            _ => {}
        }
    }*/

    for entry in data {
        //println!("{:#?}", data.kind());
        match entry.kind() {
            MesgNum::Record => {
                linearization.push(parse_fit_data_record(entry, start_time));
            }
            _ => {}
        }
    }

    linearization
}

pub fn main() -> Result<Vec<RecordDataField>, fitparser::Error> {
    println!(
        "Parsing FIT files using Profile version: {}",
        fitparser::profile::VERSION
    );

    let linearization = parse_fit_file("/home/robin/Downloads/11205437996_ACTIVITY.fit");

    for item in linearization.as_slice() {
        println!(
            "Time: {:?} Distance: {:?} Speed: {:?} {:?}",
            item.timestamp, item.distance, item.enhanced_speed, item.power
        )
    }

    Ok(linearization)
}
