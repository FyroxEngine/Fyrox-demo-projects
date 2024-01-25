use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        math::SmoothAngle,
        pool::Handle,
        reflect::prelude::*,
        type_traits::prelude::*,
        variable::InheritableVariable,
        visitor::prelude::*,
        TypeUuidProvider,
    },
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    scene::{animation::absm::prelude::*, node::Node, rigidbody::RigidBody},
    script::{ScriptContext, ScriptTrait},
};

#[derive(Visit, Reflect, Default, Debug, Clone, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "e224206c-856b-40ff-84e1-7f9bf52c2bb2")]
#[visit(optional)]
pub struct Player {
    camera_pivot: InheritableVariable<Handle<Node>>,
    camera_hinge: InheritableVariable<Handle<Node>>,
    state_machine: InheritableVariable<Handle<Node>>,
    model_pivot: InheritableVariable<Handle<Node>>,
    model: InheritableVariable<Handle<Node>>,
    model_yaw: InheritableVariable<SmoothAngle>,

    #[reflect(hidden)]
    #[visit(skip)]
    walk_forward: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    walk_backward: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    walk_left: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    walk_right: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    run: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    yaw: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    pitch: f32,
}

impl ScriptTrait for Player {
    fn on_os_event(&mut self, event: &Event<()>, ctx: &mut ScriptContext) {
        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::KeyboardInput { event, .. } = event {
                    let pressed = event.state == ElementState::Pressed;
                    if let PhysicalKey::Code(code) = event.physical_key {
                        match code {
                            KeyCode::KeyW => self.walk_forward = pressed,
                            KeyCode::KeyS => self.walk_backward = pressed,
                            KeyCode::KeyA => self.walk_left = pressed,
                            KeyCode::KeyD => self.walk_right = pressed,
                            KeyCode::ShiftLeft => self.run = pressed,
                            _ => (),
                        }
                    }
                }
            }
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    let mouse_sens = 0.2 * ctx.dt;
                    self.yaw -= (delta.0 as f32) * mouse_sens;
                    self.pitch = (self.pitch + (delta.1 as f32) * mouse_sens)
                        .clamp(-90.0f32.to_radians(), 90.0f32.to_radians());
                }
            }
            _ => (),
        }
    }

    fn on_update(&mut self, ctx: &mut ScriptContext) {
        let pivot = &ctx.scene.graph[*self.model];

        let transform = pivot.global_transform();

        let mut velocity = Vector3::default();

        if let Some(state_machine) = ctx
            .scene
            .graph
            .try_get(*self.state_machine)
            .and_then(|node| node.query_component_ref::<AnimationBlendingStateMachine>())
        {
            if let Some(root_motion) = state_machine.machine().pose().root_motion() {
                velocity = transform
                    .transform_vector(&root_motion.delta_position)
                    .scale(1.0 / ctx.dt);
            }
        }

        if let Some(body) = ctx.scene.graph.try_get_mut_of_type::<RigidBody>(ctx.handle) {
            let quat_yaw = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.yaw);

            body.set_ang_vel(Default::default());
            body.set_lin_vel(Vector3::new(velocity.x, body.lin_vel().y, velocity.z));

            if velocity.norm_squared() > 0.0 {
                // Since we have free camera while not moving, we have to sync rotation of pivot
                // with rotation of camera so character will start moving in look direction.
                if let Some(model_pivot) = ctx.scene.graph.try_get_mut(*self.model_pivot) {
                    model_pivot.local_transform_mut().set_rotation(quat_yaw);
                }

                // Apply additional rotation to model - it will turn in front of walking direction.
                let angle: f32 = if self.walk_left {
                    if self.walk_forward {
                        45.0
                    } else if self.walk_backward {
                        135.0
                    } else {
                        90.0
                    }
                } else if self.walk_right {
                    if self.walk_forward {
                        -45.0
                    } else if self.walk_backward {
                        -135.0
                    } else {
                        -90.0
                    }
                } else if self.walk_backward {
                    180.0
                } else {
                    0.0
                };

                self.model_yaw.set_target(angle.to_radians()).update(ctx.dt);

                if let Some(model) = ctx.scene.graph.try_get_mut(*self.model) {
                    model
                        .local_transform_mut()
                        .set_rotation(UnitQuaternion::from_axis_angle(
                            &Vector3::y_axis(),
                            self.model_yaw.angle,
                        ));
                }
            }

            if let Some(camera_pivot) = ctx.scene.graph.try_get_mut(*self.camera_pivot) {
                camera_pivot.local_transform_mut().set_rotation(quat_yaw);
            }

            // Rotate camera hinge - this will make camera move up and down while look at character
            // (well not exactly on character - on characters head)
            if let Some(camera_hinge) = ctx.scene.graph.try_get_mut(*self.camera_hinge) {
                camera_hinge
                    .local_transform_mut()
                    .set_rotation(UnitQuaternion::from_axis_angle(
                        &Vector3::x_axis(),
                        self.pitch,
                    ));
            }
        }

        if let Some(state_machine) = ctx
            .scene
            .graph
            .try_get_mut(*self.state_machine)
            .and_then(|node| node.query_component_mut::<AnimationBlendingStateMachine>())
        {
            let moving =
                self.walk_left || self.walk_right || self.walk_forward || self.walk_backward;

            state_machine
                .machine_mut()
                .get_value_mut_silent()
                .set_parameter("Moving", Parameter::Rule(moving))
                .set_parameter(
                    "MoveAnimationIndex",
                    Parameter::Index(if self.run { 1 } else { 0 }),
                );
        }
    }
}
