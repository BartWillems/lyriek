#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

use gdk_pixbuf::PixbufLoaderExt;

impl Assets {
    pub fn get_logo_pixbuf() -> Option<gdk_pixbuf::Pixbuf> {
        Assets::get("logo.svg")
            .and_then(|logo| {
                let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
                match pixbuf_loader.write(&logo) {
                    Ok(_) => Some(pixbuf_loader),
                    Err(e) => {
                        error!("unable to write bytes to the PixbufLoader: {}", e);
                        None
                    }
                }
            })
            .and_then(|pixbuf_loader| {
                pixbuf_loader.close().ok();
                pixbuf_loader.get_pixbuf()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_logo() {
        assert!(Assets::get_logo_pixbuf().is_some());
    }
}
