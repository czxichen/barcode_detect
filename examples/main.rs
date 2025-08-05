use image::GenericImage;

fn main() {
    const SIZE: u32 = 640;
    let mut detect = qbarcode_scanner::Detect::new();

    let src = image::open("examples/IMG_3105.jpg").unwrap();
    let ss = src
        .resize(SIZE, SIZE, image::imageops::FilterType::Lanczos3)
        .to_rgb8();
    let mut input = image::ImageBuffer::new(SIZE, SIZE);

    input.copy_from(&ss, 0, (SIZE - ss.height()) / 2).unwrap();

    let raw = input.to_vec();

    let now = std::time::SystemTime::now();
    let ds = detect.detect_with_pixels(raw.as_slice(), SIZE, 0.25);

    for d in ds.iter().enumerate() {
        println!("{:?}", d);
        // image::DynamicImage::from(input.clone())
        //     .crop(
        //         d.1.rect.x as u32,
        //         d.1.rect.y as u32,
        //         d.1.rect.w as u32,
        //         d.1.rect.h as u32,
        //     )
        //     .save(format!("examples/{}.png", d.0))
        //     .unwrap();
    }

    let ret =
        qbarcode_scanner::scan(raw.as_slice(), SIZE, SIZE, ds.as_slice()).expect("scan failed");

    println!("{:?} {:?}", now.elapsed().unwrap(), ret);
}
