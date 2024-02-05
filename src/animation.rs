use super::image;


pub fn save_gif(
    frames: Vec<Vec<u8>>,
    wh: (usize, usize),
    path: String,
) -> Result<(), failure::Error> {
    log::info!("Saving GIF...");

    let mut image = std::fs::File::create(path)?;
    let mut encoder = gif::Encoder::new(
        &mut image,
        wh.0.try_into().unwrap(),
        wh.1.try_into().unwrap(),
        &[],
    )?;
    encoder.set_repeat(gif::Repeat::Infinite)?;

    const SPEED: i32 = 1;
    for mut frame in frames {
        encoder.write_frame(
            &gif::Frame::from_rgba_speed(
                wh.0.try_into().unwrap(),
                wh.1.try_into().unwrap(),
                &mut frame,
                SPEED,
            ),
        )?;
    }

    log::info!("GIF saved.");

    Ok(())
}


pub async fn read_and_save(
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    wh: (usize, usize),
    frames_count: usize,
) {
    log::info!("Reading shader from file and saving...");

    let source = std::fs::read_to_string(input).expect("Can't read shader.");
    let frames = render(
        std::borrow::Cow::Borrowed(&source),
        wh,
        frames_count,
    ).await;
    save_gif(
        frames.to_vec(),
        wh,
        output.into_os_string().into_string().unwrap(),
    ).expect("Can't save GIF.");

    log::info!("Done.");
}


pub async fn render(
    source: std::borrow::Cow<'_, String>,
    wh: (usize, usize),
    frames_count: usize,
) -> Vec<Vec<u8>> {
    let mut frames: Vec<Vec<u8>> = vec![];
    for i in 0..frames_count {
        frames.push(image::render(source.to_string(), wh, Some(i)).await);
    }
    return frames;
}