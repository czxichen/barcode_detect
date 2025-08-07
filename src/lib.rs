#![allow(unused)]

use std::ffi::{CStr, CString, c_int, c_void};

const BMEM: &[u8] = include_bytes!("../model/scannetv2_640.ncnn.bin");
const PMEM: &[u8] = include_bytes!("../model/scannetv2_640.ncnn.param");
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

#[derive(Debug, Clone)]
#[repr(C)]
struct DecodeResult {
    pub class_id: c_int,
    pub x: c_int,
    pub y: c_int,
    pub w: c_int,
    pub h: c_int,
    text: *const i8,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct DecodeResultList {
    results: *const DecodeResult,
    count: c_int,
}

#[repr(C)]
pub struct DetectNet {
    net: *const c_void,
    input_size: c_int,
}

#[allow(unused)]
unsafe extern "C" {
    fn create_detector(param: *const i8, bin: *const u8, input_size: c_int) -> DetectNet;

    fn destroy_detector(net: *mut DetectNet);

    fn detect(
        net: *const DetectNet,
        img_p: *const u8,
        img_s: u32,
        score_threshold: f32,
        nms_threshold: f32,
        out: *mut c_int,
    ) -> *const Detection;

    // img_p: rgb 数据指针必须是正方形图片
    fn detect_with_pixels(
        net: *const DetectNet,
        img_p: *const u8,
        img_s: c_int,
        score_threshold: f32,
        nms_threshold: f32,
        out: *mut c_int,
    ) -> *const Detection;

    fn detect_result_free(ret: *const Detection);

    fn decode_detections(
        img_p: *const u8,
        width: c_int,
        height: c_int,
        detections: *const Detection,
        detections_size: c_int,
    ) -> DecodeResultList;

    fn decode_result_free(result_list: *const DecodeResultList);
}

unsafe impl Send for DetectNet {}
unsafe impl Sync for DetectNet {}

impl DetectNet {
    pub fn new(input_size: u32) -> Self {
        return unsafe {
            create_detector(
                PMEM.as_ptr() as *const _,
                BMEM.as_ptr() as *const _,
                input_size as c_int,
            )
        };
    }

    pub fn detect(&self, img: &[u8], s_threshold: f32, n_threshold: f32) -> Vec<Detection> {
        let mut out = 0i32;
        let ret = unsafe {
            detect(
                self as *const DetectNet,
                img.as_ptr(),
                img.len() as u32,
                s_threshold,
                n_threshold,
                &mut out,
            )
        };
        if out == 0 {
            return Vec::new();
        }
        let detections = unsafe { std::slice::from_raw_parts(ret, out as usize) }.to_vec();
        unsafe { detect_result_free(ret) };
        return detections;
    }

    pub fn detect_with_pixels(
        &self,
        img: &[u8],
        img_s: u32,
        s_threshold: f32,
        n_threshold: f32,
    ) -> Vec<Detection> {
        let mut out = 0i32;
        let ret = unsafe {
            detect_with_pixels(
                self,
                img.as_ptr(),
                img_s as c_int,
                s_threshold,
                n_threshold,
                &mut out,
            )
        };
        if out == 0 {
            return Vec::new();
        }
        let detections = unsafe { std::slice::from_raw_parts(ret, out as usize) }.to_vec();
        unsafe { detect_result_free(ret) };
        return detections;
    }
}

impl Drop for DetectNet {
    fn drop(&mut self) {
        unsafe { destroy_detector(self as *mut DetectNet) };
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DetectionResult {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
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
    let mut ret = Vec::new();
    if detections.is_empty() {
        return Ok(ret);
    }

    unsafe {
        let decodes = decode_detections(
            ptr.as_ptr(),
            width as c_int,
            height as c_int,
            detections.as_ptr(),
            detections.len() as c_int,
        );

        if decodes.count == 0 || decodes.results.is_null() {
            decode_result_free(&decodes);
            return Ok(ret);
        }

        for decode in std::slice::from_raw_parts(decodes.results, decodes.count as usize) {
            ret.push(DetectionResult {
                x: decode.x,
                y: decode.y,
                w: decode.w,
                h: decode.h,
                class: decode.class_id,
                codes: CStr::from_ptr(decode.text as *const _)
                    .to_string_lossy()
                    .to_string(),
            });
        }
        decode_result_free(&decodes);
    };
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
