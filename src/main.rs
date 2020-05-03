//! Displays several lines with both methods.

use amethyst::{
    controls::{FlyControlBundle, FlyControlTag},
    core::{
        math::{Point3, Vector3},
        transform::{Transform, TransformBundle},
        Time,
    },
    derive::SystemDesc,
    ecs::{Read, System, SystemData, WorldExt, Write},
    input::{is_close_requested, is_key_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        camera::{Camera, Projection},
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        palette::Srgba,
        plugins::{RenderDebugLines, RenderSkybox, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    winit::VirtualKeyCode,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::ops::DerefMut;
use glam::{Mat4, Vec3};
use physx::prelude::*;
use physx::visual_debugger::PvdSceneClient;
use physx::scene::VisualizationParameter;

pub mod color_conv;

const PX_PHYSICS_VERSION: u32 = physx::version(4, 1, 1);

#[derive(SystemDesc)]
struct ExampleLinesSystem;

impl<'s> System<'s> for ExampleLinesSystem {
    type SystemData = (
        Write<'s, DebugLines>, // Request DebugLines resource
        Read<'s, Time>,
    );

    fn run(&mut self, (mut debug_lines_resource, time): Self::SystemData) {
        // Drawing debug lines, as a resource
        let t = (time.absolute_time_seconds() as f32).cos();

        // debug_lines_resource.draw_direction(
        //     [t, 0.0, 0.5].into(),
        //     [0.0, 0.3, 0.0].into(),
        //     Srgba::new(0.5, 0.05, 0.65, 1.0),
        // );

        // debug_lines_resource.draw_line(
        //     [t, 0.0, 0.5].into(),
        //     [0.0, 0.0, 0.2].into(),
        //     Srgba::new(0.5, 0.05, 0.65, 1.0),
        // );
    }
}

const SPHERE_SIZE: f32 = 2.0;

#[derive()]
struct PhysxResources {
    pub foundation: Foundation,
    pub physics: Option<Physics>,
    pub scene: Box<Scene>,
    pub pvd_scene_client: Option<Box<PvdSceneClient>>,
    pub sphere_handle: BodyHandle
}

impl Drop for PhysxResources {
    fn drop(&mut self) {
        self.sphere_handle = BodyHandle(0);
        self.pvd_scene_client = None;
        unsafe{
            self.scene.release();
            //This calls drop implicitly
            self.physics = None;
            self.foundation.release();
        }
    }
}

#[derive(Default, Clone)]
struct PhysXRef(Option<Arc<Mutex<PhysxResources>>>);
unsafe impl Send for PhysXRef {}
unsafe impl Sync for PhysXRef {}

#[derive(SystemDesc)]
struct PhysXSystem;
impl<'a> System<'a> for PhysXSystem {
    type SystemData = (Read<'a, Time>,
        Write<'a, PhysXRef>,
        Write<'a, DebugLines>, // Request DebugLines resource
    );

    fn run(&mut self, (time, mut physx, mut debug_lines): Self::SystemData) {
        //println!("Delta time: {}", time.fixed_seconds());
        if time.delta_seconds() > 0.0 {
            let mut physx_lock = physx.0.as_ref().unwrap().lock().unwrap();
            let physx_ref = physx_lock.deref_mut();
            physx_ref.scene.simulate(time.delta_seconds());
            physx_ref.scene
                .fetch_results(true)
                .expect("error occured during simulation");

            let render_buffer = physx_ref.scene.get_render_buffer();
            for point in render_buffer.get_points() {
                let mut end_point = Point3::from(point.pos);
                end_point.x += 0.01;
                debug_lines.draw_line(Point3::from(point.pos), end_point, color_conv::unpack_color(point.color));
            }
            for line in render_buffer.get_lines() {
                debug_lines.draw_line(
                    Point3::from(line.pos0), 
                    Point3::from(line.pos1), 
                    color_conv::unpack_color(line.color0));
            }
            for triangle in render_buffer.get_triangles() {
                debug_lines.draw_line(Point3::from(triangle.pos0), Point3::from(triangle.pos1), color_conv::unpack_color(triangle.color0));
                debug_lines.draw_line(Point3::from(triangle.pos1), Point3::from(triangle.pos2), color_conv::unpack_color(triangle.color1));
                debug_lines.draw_line(Point3::from(triangle.pos0), Point3::from(triangle.pos2), color_conv::unpack_color(triangle.color2));
            }
            
            // let _ball_pos = unsafe { physx_ref.scene.get_rigid_actor_unchecked(&physx_ref.sphere_handle) }
            //     .get_global_position();        
        } 
    }

    fn dispose(self, world: &mut World)
    {
        if world.try_fetch::<PhysXRef>().is_some() {
            world.remove::<PhysXRef>();
        }
    }

}

// #[derive(SystemDesc)]
// struct RenderPhysXSystem;
// impl<'a> System<'a> for RenderPhysXSystem {
//     type SystemData = (Write<'a, Box<Scene>>);

//     fn run(&mut self, (scene): Self::SystemData) {
//         let buffer = scene.get_render_buffer();
//     }
// }

struct ExampleState{
}
impl ExampleState{
    fn new() -> ExampleState {
        ExampleState{}
    }
}
impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Setup debug lines as a resource
        data.world.insert(DebugLines::new());
        // Configure width of lines. Optional step
        data.world.insert(DebugLinesParams { line_width: 2.0 });
        

        let mut foundation = Foundation::new(PX_PHYSICS_VERSION);
        let mut physics = PhysicsBuilder::default()
            .load_extensions(false)
            .build(&mut foundation);
        let mut scene = physics.create_scene(
            SceneBuilder::default()
                .set_gravity(Vec3::new(0.0, -9.81, 0.0))
                .set_simulation_threading(SimulationThreadType::Dedicated(1)),
        );

        scene.set_visualization_parameter(VisualizationParameter::Scale, 1.0);
        scene.set_visualization_parameter(VisualizationParameter::ContactPoint, 1.0);
        scene.set_visualization_parameter(VisualizationParameter::ContactForce, 1.0);
        scene.set_visualization_parameter(VisualizationParameter::ContactNormal, 1.0);
        scene.set_visualization_parameter(VisualizationParameter::CollisionShapes, 1.0);
        scene.set_visualization_parameter(VisualizationParameter::WorldAxes, 1.0);

        let pvd_scene_client = Some(Box::new(scene.get_pvd_client()));

        let material = physics.create_material(0.5, 0.5, 0.6);
        let ground_plane = unsafe { physics.create_plane(Vec3::new(0.0, 1.0, 0.0), 0.0, material) };
        scene.add_actor(ground_plane);
    
        let sphere_geo = PhysicsGeometry::from(&ColliderDesc::Sphere(SPHERE_SIZE));
        let mut sphere_actor = unsafe {
            physics.create_dynamic(
                Mat4::from_translation(Vec3::new(1.0, 40.0, -4.0)),
                sphere_geo.as_raw(), // todo: this should take the PhysicsGeometry straight.
                material,
                10.0,
                Mat4::identity(),
            )
        };
    
        sphere_actor.set_angular_damping(0.5);
        let sphere_handle = scene.add_dynamic(sphere_actor);
        let physics_resources = PhysxResources{foundation, physics: Some(physics), scene, pvd_scene_client, sphere_handle};

        data.world.insert(PhysXRef( Some(Arc::new(Mutex::new(physics_resources))) ));

        // Setup debug lines as a component and add lines to render axis&grid
        let mut debug_lines_component = DebugLinesComponent::with_capacity(100);
        // debug_lines_component.add_direction(
        //     [0.0, 0.0001, 0.0].into(),
        //     [0.2, 0.0, 0.0].into(),
        //     Srgba::new(1.0, 0.0, 0.23, 1.0),
        // );
        // debug_lines_component.add_direction(
        //     [0.0, 0.0, 0.0].into(),
        //     [0.0, 0.2, 0.0].into(),
        //     Srgba::new(0.5, 0.85, 0.1, 1.0),
        // );
        // debug_lines_component.add_direction(
        //     [0.0, 0.0001, 0.0].into(),
        //     [0.0, 0.0, 0.2].into(),
        //     Srgba::new(0.2, 0.75, 0.93, 1.0),
        // );

        let width: u32 = 10;
        let depth: u32 = 10;
        let main_color = Srgba::new(0.4, 0.4, 0.4, 1.0);

        // Grid lines in X-axis
        for x in 0..=width {
            let (x, width, depth) = (x as f32, width as f32, depth as f32);

            let position = Point3::new(x - width / 2.0, 0.0, -depth / 2.0);
            let direction = Vector3::new(0.0, 0.0, depth);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if (x - width).abs() < 0.0001 {
                for sub_x in 1..10 {
                    let sub_offset = Vector3::new((1.0 / 10.0) * sub_x as f32, -0.001, 0.0);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        Srgba::new(0.1, 0.1, 0.1, 0.2),
                    );
                }
            }
        }

        // Grid lines in Z-axis
        for z in 0..=depth {
            let (z, width, depth) = (z as f32, width as f32, depth as f32);

            let position = Point3::new(-width / 2.0, 0.0, z - depth / 2.0);
            let direction = Vector3::new(width, 0.0, 0.0);

            debug_lines_component.add_direction(position, direction, main_color);

            // Sub-grid lines
            if (z - depth).abs() < 0.0001 {
                for sub_z in 1..10 {
                    let sub_offset = Vector3::new(0.0, -0.001, (1.0 / 10.0) * sub_z as f32);

                    debug_lines_component.add_direction(
                        position + sub_offset,
                        direction,
                        Srgba::new(0.1, 0.1, 0.1, 0.2),
                    );
                }
            }
        }
        data.world.register::<DebugLinesComponent>();
        data.world
            .create_entity()
            .with(debug_lines_component)
            .build();

        // Setup camera
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(0.0, 0.5, 2.0);
        data.world
            .create_entity()
            .with(FlyControlTag)
            .with(Camera::from(Projection::perspective(
                1.33333,
                std::f32::consts::FRAC_PI_2,
                0.1,
                1000.0,
            )))
            .with(local_transform)
            .build();
    

    
        // unsafe {
        //     scene.release();
        // }
    
        // drop(physics);
    
        // foundation.release();
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }

    // fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        

    //     Trans::None
    // }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("config/display.ron");
    let key_bindings_path = app_root.join("config/input.ron");
    let assets_dir = app_root.join("assets/");

    let fly_control_bundle = FlyControlBundle::<StringBindings>::new(
        Some(String::from("move_x")),
        Some(String::from("move_y")),
        Some(String::from("move_z")),
    )
    .with_sensitivity(0.1, 0.1);

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with(ExampleLinesSystem, "example_lines_system", &[])
        .with(PhysXSystem, "PhysX system", &[])
        .with_bundle(fly_control_bundle)?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderSkybox::default()),
        )?;

    let mut game = Application::new(assets_dir, ExampleState::new(), game_data)?;
    game.run();
    Ok(())
}
