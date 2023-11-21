#define_import_path bevy_entitiles::height_map

#ifdef RENDER_STAGE
@group(3) @binding(0)
var height_map: texture_storage_2d<rgba8unorm, read_write>;
#else
@group(1) @binding(0)
var height_map: texture_storage_2d<rgba8unorm, read_write>;
#endif
