use gdk_pixbuf::PixbufLoaderExt;

pub fn get_logo_pixbuf() -> Option<gdk_pixbuf::Pixbuf> {
    let logo_bytes = include_bytes!("../assets/logo.svg");

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();

    let pixbuf_loader = match pixbuf_loader.write(logo_bytes) {
        Ok(_) => Some(pixbuf_loader),
        Err(e) => {
            error!("unable to write bytes to the PixbufLoader: {}", e);
            return None;
        }
    }?;

    pixbuf_loader.close().ok();
    pixbuf_loader.get_pixbuf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_logo() {
        assert!(get_logo_pixbuf().is_some());
    }
}
