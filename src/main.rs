mod parser;

use std::ptr::{null, null_mut};

use chrono::{offset, DateTime, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::env;
use std::fmt::Error;

use opencv::core::{Point, Scalar, Size, Vector, BORDER_DEFAULT};
use opencv::gapi::GMat;
use opencv::imgproc::put_text;
use opencv::imgproc::INTER_LINEAR;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureProperties, VideoWriter, CAP_ANY, CAP_FFMPEG};
use opencv::{gapi, highgui, Result};

fn draw_text(img: &mut Mat, text: &str, line: i32, frame_height: i32) -> Result<()> {
    let font = 1;
    let scale = 5.0;
    let color = Scalar::new(255., 255., 255., 0.);
    let thickness = 3;
    let line_type = 8;
    let bottom_left_origin = false;

    put_text(
        img,
        text,
        Point::new(10, frame_height - (80 * line)),
        font,
        scale,
        color,
        thickness,
        line_type,
        bottom_left_origin,
    )?;

    Ok(())
}

pub fn main() -> Result<(), opencv::Error> {
    let linearization = parser::main().expect("Failed to read");

    let mut tracking_index = 0i64;

    let video_file_path = "/run/media/robin/disk/DCIM/100_CF12/CYQ_0368.MP4";

    let mut cap = VideoCapture::from_file(video_file_path, CAP_FFMPEG)?;
    let video_ffprobe = match ffprobe::ffprobe(video_file_path) {
        Ok(v) => dbg!(v),
        Err(e) => panic!("ffprobe error: {}", e),
    };

    let creation_time_string = match video_ffprobe.streams[0].tags.as_ref() {
        Some(tags) => tags.creation_time.as_ref().unwrap(),
        None => panic!("No creation time"),
    };

    let creation_time: NaiveDateTime =
        NaiveDateTime::parse_from_str(creation_time_string, "%Y-%m-%dT%H:%M:%S%.fZ").unwrap();

    println!("creation_time: {}", creation_time);

    let mut current_track_record = &linearization[tracking_index as usize];

    println!("First track time: {:?}", current_track_record.timestamp);

    let mut next_track_record = &linearization[(tracking_index + 1) as usize];

    while creation_time > current_track_record.timestamp
        && tracking_index < linearization.len() as i64 - 1
    {
        current_track_record = &linearization[tracking_index as usize];
        next_track_record = &linearization[(tracking_index + 1) as usize];

        tracking_index += 1;
    }

    print!("Last track time: {:?}", current_track_record.timestamp);

    if tracking_index == linearization.len() as i64 - 1 {
        panic!("No track record found");
    }

    let fps = cap.get(5).unwrap();
    let frame_height = cap.get(4).unwrap() as i32;
    let frame_width = cap.get(3).unwrap() as i32;

    let mut video_writer = VideoWriter::new(
        "/home/robin/test.mkv",
        VideoWriter::fourcc('h', '2', '6', '4').unwrap().into(),
        fps,
        Size::new(frame_width, frame_height),
        true,
    )?;

    let mut frame_num = 0;
    let mut last_track_time = 0f64;

    let frame_count = cap.get(7).unwrap() as i32;

    loop {
        frame_num += 1;

        let time_offset = frame_num as f64 / fps;

        if creation_time + Duration::seconds(time_offset as i64) >= next_track_record.timestamp {
            tracking_index += 1;

            current_track_record = &linearization[tracking_index as usize];
            next_track_record = &linearization[(tracking_index + 1) as usize];

            last_track_time = time_offset;
        }

        //println!("Frame: {} time: {}", frame_num, time_offset);

        let mut input_frame = Mat::default();
        let has_next = cap.read(&mut input_frame)?;

        if !has_next {
            break;
        }

        let time_delta_seconds = (next_track_record.timestamp - current_track_record.timestamp)
            .num_microseconds()
            .unwrap() as f64
            * 0.000001;

        let current_power = current_track_record.power.unwrap_or(0) as f64;
        let next_power = next_track_record.power.unwrap_or(0) as f64;
        let power_delta = next_power - current_power;

        // Scale linearly between current and next power
        let power =
            current_power + power_delta * ((time_offset - last_track_time) / time_delta_seconds);

        let current_speed = current_track_record.enhanced_speed * 3.6;
        let next_speed = next_track_record.enhanced_speed * 3.6;
        let speed_delta = next_speed - current_speed;

        let speed = current_speed + speed_delta * ((time_offset - last_track_time) / time_delta_seconds);

        let heart_rate = current_track_record.hr.unwrap_or(0);

        draw_text(
            &mut input_frame,
            format!("Power: {:.2} watts", power).as_str(),
            1,
            frame_height,
        )?;

        draw_text(
            &mut input_frame,
            format!("Heart Rate: {} bpm", heart_rate).as_str(),
            2,
            frame_height,
        )?;

        draw_text(
            &mut input_frame,
            format!("Speed: {:.2} km/h", speed).as_str(),
            3,
            frame_height,
        )?;
        video_writer.write(&input_frame)?;
    }

    video_writer.release()?;

    Ok(())
}
