use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        variable::InheritableVariable,
        visitor::prelude::*,
        TypeUuidProvider,
    },
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    impl_component_provider,
    keyboard::KeyCode,
    scene::node::Node,
    script::{ScriptContext, ScriptTrait},
};

#[derive(Visit, Reflect, Default, Debug, Clone)]
pub struct CameraController {
    camera: InheritableVariable<Handle<Node>>,
    #[reflect(hidden)]
    #[visit(skip)]
    move_forward: bool,
    #[reflect(hidden)]
    #[visit(skip)]
    move_backward: bool,
    #[reflect(hidden)]
    #[visit(skip)]
    move_left: bool,
    #[reflect(hidden)]
    #[visit(skip)]
    move_right: bool,
    #[reflect(hidden)]
    #[visit(skip)]
    yaw: f32,
    #[reflect(hidden)]
    #[visit(skip)]
    pitch: f32,
}

impl_component_provider!(CameraController);

impl TypeUuidProvider for CameraController {
    fn type_uuid() -> Uuid {
        uuid!("8d9e2feb-8c61-482c-8ba4-b0b13b201113")
    }
}

impl ScriptTrait for CameraController {
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::KeyboardInput { event, .. } = event {
                    let pressed = event.state == ElementState::Pressed;
                    match event.physical_key {
                        KeyCode::KeyW => {
                            self.move_forward = pressed;
                        }
                        KeyCode::KeyS => {
                            self.move_backward = pressed;
                        }
                        KeyCode::KeyA => {
                            self.move_left = pressed;
                        }
                        KeyCode::KeyD => {
                            self.move_right = pressed;
                        }
                        _ => (),
                    }
                }
            }
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta, .. } = event {
                    let speed = 0.7 * context.dt;
                    self.yaw -= (delta.0 as f32) * speed;
                    self.pitch = (self.pitch + delta.1 as f32 * speed)
                        .clamp(-89.9f32.to_radians(), 89.9f32.to_radians());
                }
            }
            _ => {}
        }
    }

    fn on_update(&mut self, context: &mut ScriptContext) {
        if let Some(pivot) = context.scene.graph.try_get_mut(*self.camera) {
            pivot
                .local_transform_mut()
                .set_rotation(UnitQuaternion::from_axis_angle(
                    &Vector3::x_axis(),
                    self.pitch,
                ));
        }

        let this = &mut context.scene.graph[context.handle];

        this.local_transform_mut()
            .set_rotation(UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                self.yaw,
            ));

        let mut velocity = Vector3::default();
        if self.move_forward {
            velocity += this.look_vector();
        }
        if self.move_backward {
            velocity -= this.look_vector();
        }
        if self.move_left {
            velocity += this.side_vector();
        }
        if self.move_right {
            velocity -= this.side_vector();
        }
        if let Some(normalized_velocity) = velocity.try_normalize(f32::EPSILON) {
            this.local_transform_mut()
                .offset(normalized_velocity.scale(5.0 * context.dt));
        }
    }

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}
