use battle_core::map::Map;
use ggez::{
    graphics::{Image, InstanceArray},
    Context, GameError, GameResult,
};
use image::imageops::{resize, FilterType};
use oc_core::resources::Resources;

use crate::ui::hud::minimap::{MINIMAP_HEIGHT, MINIMAP_WIDTH};

pub struct MinimapBuilder<'a> {
    ctx: &'a mut Context,
    map: &'a Map,
}

impl<'a> MinimapBuilder<'a> {
    pub fn new(ctx: &'a mut Context, map: &'a Map) -> Self {
        Self { ctx, map }
    }

    pub fn build(&self) -> GameResult<InstanceArray> {
        let resources = match Resources::new() {
            Ok(resources) => resources,
            Err(error) => return Err(GameError::ResourceLoadError(error.to_string())),
        };
        let bg_image_path_abs = resources.lib().join(
            self.map
                .background_image_path()
                .strip_prefix("/")
                .expect("Must start with /"),
        );
        let minimap_path_cache_abs = resources
            .cache_abs()
            .join(format!("{}__minimap.png", self.map.name()));

        let bg_dark_image_path_rel = resources
            .cache_ggez()
            .join(format!("{}__minimap.png", self.map.name()));

        if !minimap_path_cache_abs.exists() {
            let bg_image = image::open(bg_image_path_abs)?.into_rgba8();
            let minimap = resize(
                &bg_image,
                // TODO : use arbitrary size and draw at wanted size
                MINIMAP_WIDTH as u32,
                MINIMAP_HEIGHT as u32,
                FilterType::Gaussian,
            );
            minimap.save(minimap_path_cache_abs)?;
        }

        let minimap = Image::from_path(self.ctx, bg_dark_image_path_rel)?;
        let instance_array = InstanceArray::new(self.ctx, minimap);
        Ok(instance_array)
    }
}
