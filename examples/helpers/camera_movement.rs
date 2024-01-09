use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Resource)]
pub struct CameraControl {
    pub target_pos: Vec2,
    pub target_scale: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            target_pos: Default::default(),
            target_scale: 1.,
        }
    }
}

pub fn camera_control(
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
    input_keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    mut event_wheel: EventReader<MouseWheel>,
    mut event_move: EventReader<MouseMotion>,
    time: Res<Time>,
    mut control: ResMut<CameraControl>,
) {
    let (mut transform, mut projection) = query.get_single_mut().unwrap();

    if input_mouse.pressed(MouseButton::Left) {
        for ev in event_move.read() {
            control.target_pos +=
                200. * projection.scale * time.delta_seconds() * ev.delta * Vec2::new(-1., 1.);
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
        control.target_pos += Vec2::new(x as f32 * step, 0.);

        let mut y = 0;
        if input_keyboard.pressed(KeyCode::W) {
            y += 1;
        }
        if input_keyboard.pressed(KeyCode::S) {
            y -= 1;
        }
        control.target_pos += y as f32 * step * Vec2::Y;
    }

    let target = control.target_pos.extend(0.);
    if transform.translation.distance_squared(target) > 0.01 {
        transform.translation = transform
            .translation
            .lerp(target, 40. * time.delta_seconds());
    }

    for ev in event_wheel.read() {
        control.target_scale -= ev.y * 0.02;
        control.target_scale = control.target_scale.max(0.01);
    }

    if (projection.scale - control.target_scale).abs() > 0.01 {
        projection.scale = projection.scale
            + ((control.target_scale - projection.scale) * 20. * time.delta_seconds());
    }
    event_move.clear();
}
