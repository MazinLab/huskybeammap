use macroquad::prelude::*;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, facet::Facet)]
enum Draw {
    Milo,
    Dvd,
    Rectangle { width: usize, height: usize },
}
#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
struct Movement {
    position: isize,
    pixels: isize,
    frames: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
struct Object {
    start: Option<u64>,
    lifetime: u64,
    draw: Draw,
    x: Movement,
    y: Movement,
}

#[derive(Clone, Debug, facet::Facet)]
struct StatusMessage {
    width: usize,
    height: usize,
    frame: u64,
    objects: usize,
}

impl Object {
    fn position(&self, frame: u64) -> (isize, isize) {
        let start = self.start.unwrap_or(frame);
        let start = if start < frame { start } else { frame };
        let dx = ((frame - start) / self.x.frames) as isize * self.x.pixels;
        let dy = ((frame - start) / self.y.frames) as isize * self.y.pixels;
        (self.x.position + dx, self.y.position + dy)
    }

    fn repack(self, frame: u64) -> Option<Self> {
        let repacked = Object {
            start: Some(self.start.unwrap_or(frame)),
            lifetime: self.lifetime,
            draw: self.draw,
            x: self.x,
            y: self.y,
        };

        if frame >= repacked.start.unwrap() + repacked.lifetime {
            None
        } else {
            Some(repacked)
        }
    }
}

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
