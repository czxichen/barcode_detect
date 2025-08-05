#![allow(unused)]

use std::ffi::{c_int, c_void};
use zxingcpp::{Barcode, BarcodeFormat};

const BMEM: &[u8] = include_bytes!("../model/scanner.ncnn.bin");
const PMEM: &[u8] = include_bytes!("../model/scanner.ncnn.param");
const CLASSES: [&str; 2] = ["barcode", "qrcode"];

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Rect {
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Detection {
    pub rect: Rect,
    pub score: f32,
    pub class_id: c_int,
}

impl Detection {
    pub fn class(&self) -> String {
        if self.class_id > 1 {
            return "".to_string();
        }
        return CLASSES[self.class_id as usize].to_string();
    }
}

#[allow(unused)]
unsafe extern "C" {
    fn create_detector(param: *const i8, bin: *const u8) -> *const c_void;

    fn destroy_detector(net: *const c_void);

    fn detect(
        handle: *const c_void,
        img_p: *const u8,
        img_s: u32,
        threshold: f32,
        out: *mut c_int,
    ) -> *const Detection;

    // img_p: rgb 数据指针必须是正方形图片
    fn detect_with_pixels(
        handle: *const c_void,
        img_p: *const u8,
        img_s: c_int,
        threshold: f32,
        out: *mut c_int,
    ) -> *const Detection;

    fn detect_result_free(ret: *const Detection);
}

pub struct Detect {
    handle: *const c_void,
}

impl Detect {
    pub fn new() -> Self {
        return Detect {
            handle: unsafe {
                create_detector(PMEM.as_ptr() as *const _, BMEM.as_ptr() as *const _)
            },
        };
    }

    pub fn detect(&mut self, img: &[u8], th: f32) -> Vec<Detection> {
        let mut out = 0i32;
        let ret = unsafe { detect(self.handle, img.as_ptr(), img.len() as u32, th, &mut out) };
        if out == 0 {
            return Vec::new();
        }
        let detections = unsafe { std::slice::from_raw_parts(ret, out as usize) }.to_vec();
        unsafe { detect_result_free(ret) };
        return detections;
    }

    pub fn detect_with_pixels(&mut self, img: &[u8], img_s: u32, th: f32) -> Vec<Detection> {
        let mut out = 0i32;
        let ret =
            unsafe { detect_with_pixels(self.handle, img.as_ptr(), img_s as c_int, th, &mut out) };
        if out == 0 {
            return Vec::new();
        }
        let detections = unsafe { std::slice::from_raw_parts(ret, out as usize) }.to_vec();
        unsafe { detect_result_free(ret) };
        return detections;
    }
}

impl Drop for Detect {
    fn drop(&mut self) {
        unsafe { destroy_detector(self.handle) };
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DetectionResult {
    pub l: i32,
    pub r: i32,
    pub t: i32,
    pub b: i32,
    pub class: i32,
    pub codes: String,
}

#[allow(unused)]
pub fn scan(
    ptr: &[u8],
    width: u32,
    height: u32,
    detections: &[Detection],
) -> Result<Vec<DetectionResult>, String> {
    let view = zxingcpp::ImageView::from_slice(ptr, width, height, zxingcpp::ImageFormat::RGB)
        .map_err(|err| err.to_string())?;

    let qrcode = zxingcpp::read()
        .formats(BarcodeFormat::MatrixCodes)
        .try_invert(false);

    let barcode = zxingcpp::read()
        .formats(BarcodeFormat::LinearCodes)
        .try_invert(false);

    let mut ret = Vec::with_capacity(detections.len());
    for detection in detections {
        let codes = match detection.class_id {
            0 => barcode
                .from(view.clone().cropped(
                    detection.rect.x,
                    detection.rect.y,
                    detection.rect.w,
                    detection.rect.h,
                ))
                .unwrap_or(Vec::new()),
            1 => qrcode
                .from(view.clone().cropped(
                    detection.rect.x,
                    detection.rect.y,
                    detection.rect.w,
                    detection.rect.h,
                ))
                .unwrap_or(Vec::new()),
            _ => continue,
        };

        if codes.is_empty() {
            continue;
        }

        for code in codes {
            let pos = code.position();
            ret.push(DetectionResult {
                t: pos.top_left.y.min(pos.top_right.y) + detection.rect.y as i32,
                b: pos.bottom_left.y.max(pos.bottom_right.y) + detection.rect.y as i32,
                l: pos.top_left.x.min(pos.top_left.x) + detection.rect.x as i32,
                r: pos.bottom_right.x.max(pos.top_right.x) + detection.rect.x as i32,
                class: detection.class_id,
                codes: code.text(),
            });
        }
    }
    return Ok(ret);
}

#[test]
fn test_scan() {
    let mut img = image::open("IMG_3106.jpg").unwrap();
    let img_x = img.to_rgb8();
    let now = std::time::SystemTime::now();
    let ret = scan(
        &img_x.to_vec(),
        img.width(),
        img.height(),
        &vec![Detection {
            rect: Rect {
                x: 796,
                y: 780,
                w: 1646 - 796,
                h: 1165 - 780,
            },
            class_id: 0,
            score: 0.32,
        }],
    )
    .unwrap();
    println!("{:?}", now.elapsed().unwrap());
}
