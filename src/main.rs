use std::{env, fs};

use image::EncodableLayout;
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::{
        events::room::message::{MessageType, SyncRoomMessageEvent},
        UserId,
    },
    Client, attachment::AttachmentConfig,
};
use rand::{rngs::StdRng, SeedableRng, Rng};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let user_id_env = env::var("UID_MATRIX").expect("PW supplied");
    let user_id = <&UserId>::try_from(user_id_env.as_str()).expect("parse user id");
    let pw = env::var("PW_MATRIX").expect("PW supplied");
    let client = Client::builder().user_id(user_id).build().await?;
    client.login_username(user_id, &pw).send().await?;

    let command_name = env::var("MATRIX_COMMAND").expect("MATRIX_COMMAND supplied");
    let cmd = format!("!{}", command_name);
    let folder = env::var("RANDOM_FOLDER").expect("RANDOM_FOLDER supplied");
    let paths = fs::read_dir(&folder).unwrap();
    let mut images = vec![];
    for path in paths {
        let path: fs::DirEntry = path.expect("path");
        images.push(path.file_name().as_os_str().to_str().unwrap().to_string())
    }
    let images = images.to_owned();
    client.add_event_handler({
        move |ev: SyncRoomMessageEvent, room: Room| {
            let cmd = cmd.clone();
            let images = images.clone();
            let folder = folder.clone();
            async move {
                if let Room::Joined(room) = room {
                    let MessageType::Text(ref text_content) = ev.as_original().expect("Get evt").content.msgtype else {
                        return;
                    };
                    if text_content.body.contains(&cmd) {
                        let mut rnd: StdRng = StdRng::from_entropy();
                        let range = rnd.gen_range(0..=images.len());
                        println!("{}", range);
                        let image = images.get(range).unwrap();
                        let path = format!("{}/{}", folder, image);
                        let img = fs::read(path).unwrap();
                        let fmt = match image {
                            i if i.contains("jpg") => &mime::IMAGE_JPEG,
                            i if i.contains("png") => &mime::IMAGE_PNG,
                            i if i.contains("gif") => &mime::IMAGE_GIF,
                            _ => &mime::IMAGE_JPEG,
                        };
                        room.send_attachment("",  fmt, img.as_bytes(), AttachmentConfig::new()).await.unwrap();
                    }
                }
            }
        }
    });
    client.sync(SyncSettings::default()).await?;
    Ok(())
}
