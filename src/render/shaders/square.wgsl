#define_import_path bevy_entitiles::square

#import bevy_entitiles::common MeshOutput

fn get_mesh(v_index: u32, v_pos_ws: vec3<f32>) -> MeshOutput {
    let i = v_index % 4u;
}
