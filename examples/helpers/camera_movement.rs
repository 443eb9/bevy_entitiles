use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

static mut CAMERA_TARGET_POSITION: Vec3 = Vec3::ZERO;
static mut CAMERA_TARGET_SCALE: f32 = 0.1;

pub fn camera_control(
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
    input_keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    mut event_wheel: EventReader<MouseWheel>,
    mut event_move: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let (mut transform, mut projection) = query.get_single_mut().unwrap();

    unsafe {
        // only accepts mouse or keyboard input
        if input_mouse.pressed(MouseButton::Left) {
            for ev in event_move.read() {
                CAMERA_TARGET_POSITION +=
                    1. * projection.scale * ev.delta.extend(0.) * Vec3::new(-1., 1., 0.);
            }
        } else {
            let mut step = 90. * time.delta_seconds();
            if input_keyboard.pressed(KeyCode::ShiftLeft) {
                step *= 2.;
            }

            let mut x = 0;
            if input_keyboard.pressed(KeyCode::D) {
                x += 1;
            }
            if input_keyboard.pressed(KeyCode::A) {
                x -= 1;
            }
            CAMERA_TARGET_POSITION += x as f32 * step * Vec3::X;

            let mut y = 0;
            if input_keyboard.pressed(KeyCode::W) {
                y += 1;
            }
            if input_keyboard.pressed(KeyCode::S) {
                y -= 1;
            }
            CAMERA_TARGET_POSITION += y as f32 * step * Vec3::Y;
        }

        if transform
            .translation
            .distance_squared(CAMERA_TARGET_POSITION)
            > 0.01
        {
            transform.translation = transform
                .translation
                .lerp(CAMERA_TARGET_POSITION, 40. * time.delta_seconds());
        }

        for ev in event_wheel.read() {
            CAMERA_TARGET_SCALE -= ev.y * 0.02;
            CAMERA_TARGET_SCALE = CAMERA_TARGET_SCALE.max(0.01);
        }

        if (projection.scale - CAMERA_TARGET_SCALE).abs() > 0.01 {
            projection.scale = projection.scale
                + ((CAMERA_TARGET_SCALE - projection.scale) * 20. * time.delta_seconds());
        }
    }
}
