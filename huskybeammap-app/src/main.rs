use huskybeammap_types::*;
use macroquad::prelude::*;

#[macroquad::main("Husky Beam Map")]
async fn main() {
    #[cfg(target_arch = "wasm32")]
    sapp_console_log::init().unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    let dvd = std::include_bytes!("../assets/dvd.png");
    let dvd_texture = Texture2D::from_file_with_format(dvd, Some(ImageFormat::Png));
    let milo = std::include_bytes!("../assets/milo.png");
    let milo_texture = Texture2D::from_file_with_format(milo, Some(ImageFormat::Png));
    let mut f = 0u64;

    //TODO: macroquad::window::set_fullscreen(true);
    //TODO: getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);

    let mut objects = vec![];

    let mut res = ewebsock::connect("ws://192.168.42.144:9001", Default::default());
    if let Err(e) = &res {
        error!("Couldn't connect to websocket {:#?}", e)
    }

    loop {
        if let Ok((s, r)) = &mut res {
            if let Some(event) = r.try_recv() {
                if let ewebsock::WsEvent::Message(ewebsock::WsMessage::Text(jsdata)) = &event {
                    trace!("Recieved Text: {}", jsdata);
                    let data: Vec<Object> = facet_json::from_str(jsdata).unwrap();
                    for obj in data.into_iter() {
                        objects.push(obj)
                    }
                    let resp = StatusMessage {
                        width: screen_width() as usize,
                        height: screen_height() as usize,
                        frame: f,
                        objects: objects.len(),
                        frame_time: macroquad::time::get_frame_time(),
                    };
                    s.send(ewebsock::WsMessage::Text(facet_json::to_string(&resp)));
                } else {
                    error!("Recieved unknown message {:#?}", event);
                }
            }
        }
        objects = objects.into_iter().filter_map(|p| p.repack(f)).collect();
        for o in objects.iter() {
            let pos = o.position(f);
            if f >= o.start.unwrap_or(f) {
                match o.draw {
                    Draw::Rectangle { width, height } => {
                        draw_rectangle(
                            pos.0 as f32,
                            pos.1 as f32,
                            width as f32,
                            height as f32,
                            WHITE,
                        );
                    }
                    Draw::Milo => {
                        draw_texture(&milo_texture, pos.0 as f32, pos.1 as f32, WHITE);
                    }
                    Draw::Dvd => {
                        draw_texture(&dvd_texture, pos.0 as f32, pos.1 as f32, WHITE);
                    }
                }
            }
        }

        f += 1;

        next_frame().await
    }
}
