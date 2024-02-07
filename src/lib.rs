#![feature(async_fn_in_trait)]


use std::sync::Arc;

use clipboard::{ClipboardProvider, ClipboardContext};
use ellipsoid::prelude::{
    winit::event::{ElementState, VirtualKeyCode},
    *, egui::Label,
};
use stellar_bit_core::prelude::*;

const SPACECRAFT_Z: f32 = 0.2;
const BACKGROUND_Z: f32 = 0.99;

#[repr(u32)]
#[derive(Default, Clone, Copy, strum::Display, strum::EnumIter, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum SpacecraftTextures {
    #[default]
    White,
    BlockComponent,
    LaserWeaponComponent,
    MissileWeaponComponent,
    RaptorEngineComponent,
    CentralComponent
}

impl Into<u32> for SpacecraftTextures {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Textures for SpacecraftTextures {}

type Txts = SpacecraftTextures;

pub struct SpacecraftBuilderApp {
    pub graphics: Graphics<Txts>,
    spacecraft_structure: SpacecraftStructure,
    selected_component: Option<ComponentType>,
    zoom: f32,
    mouse_pos: Vec2,
    orientation: Orientation,
    json_text_selected: bool
}

impl App<Txts> for SpacecraftBuilderApp {
    async fn new(window: winit::window::Window) -> Self {
        let graphics = Graphics::new(window).await;
        Self {
            graphics,
            spacecraft_structure: SpacecraftStructure::new(),
            selected_component: None,
            zoom: 0.15,
            mouse_pos: vec2(0., 0.),
            orientation: Orientation::Up,
            json_text_selected: false
        }
    }

    fn graphics(&self) -> &Graphics<Txts> {
        &self.graphics
    }

    fn graphics_mut(&mut self) -> &mut Graphics<Txts> {
        &mut self.graphics
    }

    fn update(&mut self, dt: f32) {}

    fn draw(&mut self) {
        let camera = GTransform::from_inflation(self.zoom);

        self.draw_spacecraft_structure(camera);
        self.draw_extras(camera);
        self.draw_background();
        self.draw_menus();
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::MouseInput { state, button, .. } = event {
            if state == &winit::event::ElementState::Pressed {
                match button {
                    winit::event::MouseButton::Left => {
                        if let Some(placeholder) = self.selected_component_as_placeholder() {
                            self.spacecraft_structure
                                .component_placeholders
                                .push(placeholder);
                        }
                    }
                    winit::event::MouseButton::Right => {
                        if self.selected_component.is_some() {
                            self.selected_component = None;
                        }
                        else {
                            let hovered_component = self.spacecraft_structure.component_placeholders.iter().position(|placholder| {
                                placholder.position == self.mouse_cell_pos()
                            });
                            if let Some(index) = hovered_component {
                                self.spacecraft_structure.component_placeholders.remove(index);
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if let WindowEvent::CursorMoved { position, .. } = event {
            let x = position.x as f32 / self.graphics.window().inner_size().width as f32;
            let y = position.y as f32 / self.graphics.window().inner_size().height as f32;

            self.mouse_pos = vec2(x, -y) * 2. - vec2(1., -1.);
        } else if let WindowEvent::KeyboardInput { input, .. } = event {
            if let Some(VirtualKeyCode::R) = input.virtual_keycode {
                if input.state == ElementState::Pressed {
                    self.orientation = self.orientation.next();
                }
            }
        }
        false
    }
}

fn component_shape(world_gtransform: GTransform, component: &ComponentPlaceholder) -> Shape<Txts> {
    let gtransform = world_gtransform
        .translate(component.position.as_vec2())
        .rotate(component.orientation.to_radians())
        .translate(-Vec2::ONE * 0.5)
        .stretch(component.component_type.scale().as_vec2());

    let texture = match component.component_type {
        ComponentType::LaserWeapon => SpacecraftTextures::LaserWeaponComponent,
        ComponentType::Central => SpacecraftTextures::CentralComponent,
        ComponentType::MissileLauncher => SpacecraftTextures::MissileWeaponComponent,
        ComponentType::RaptorEngine => SpacecraftTextures::RaptorEngineComponent,
        ComponentType::SteelBlock => SpacecraftTextures::BlockComponent,
    };

    Shape::from_square()
        .apply(gtransform)
        .set_texture(texture)
        .set_z(if component.component_type.top().is_none() {
            SPACECRAFT_Z
        } else {
            SPACECRAFT_Z - 0.01
        })
}

impl SpacecraftBuilderApp {
    fn selected_component_as_placeholder(&self) -> Option<ComponentPlaceholder> {
        self.selected_component.map(|component| {
            ComponentPlaceholder::new(component, self.mouse_cell_pos(), self.orientation)
        })
    }
    fn mouse_cell_pos(&self) -> IVec2 {
        (self.mouse_pos * 1. / self.zoom + Vec2::ONE * 0.5)
            .floor()
            .as_ivec2()
    }
    fn draw_spacecraft_structure(&mut self, camera: GTransform) {
        for component in &self.spacecraft_structure.component_placeholders {
            let shape = component_shape(camera, component);
            self.graphics.add_geometry(shape.into());
        }
    }
    fn draw_extras(&mut self, camera: GTransform) {
        if let Some(placeholder) = self.selected_component_as_placeholder() {
            let shape = component_shape(camera, &placeholder).set_color(Color::from_rgba(1., 1., 1., 0.5));

            self.graphics.add_geometry(shape.into());
        }
    }
    fn draw_background(&mut self) {
        let background_color = if self.spacecraft_structure.valid() {
            Color::from_hex(0x222222)
        } else {
            Color::from_hex(0xaa1111)
        };
        let background_shape = Shape::from_square_centered()
            .set_color(background_color)
            .set_z(BACKGROUND_Z)
            .apply(GTransform::from_inflation(2.));
        self.graphics.add_geometry(background_shape.into());

        let line_width = 0.003;
        let width = 11;
        let line_color = 0x888888;
        for i in 0..width - 1 {
            let x_pos = i as f32 - (width - 2) as f32 * 0.5;
            let line_gtransform = GTransform::from_inflation(self.zoom)
                .translate(Vec2::X * x_pos)
                .set_scale(vec2(line_width, 1.9));
            let line_shape = Shape::from_square_centered()
                .apply(line_gtransform)
                .set_z(BACKGROUND_Z - 0.1)
                .set_color(Color::from_hex(line_color));

            self.graphics.add_geometry(line_shape.into());
        }
        let height = 11;
        for i in 0..height - 1 {
            let y_pos = i as f32 - (height - 2) as f32 * 0.5;
            let line_gtransform = GTransform::from_inflation(self.zoom)
                .translate(Vec2::Y * y_pos)
                .set_scale(vec2(1.9, line_width));
            let line_shape = Shape::from_square_centered()
                .apply(line_gtransform)
                .set_z(BACKGROUND_Z - 0.1)
                .set_color(Color::from_hex(line_color));

            self.graphics.add_geometry(line_shape.into());
        }
    }
    fn draw_menus(&mut self) {
        let weapons = vec![ComponentType::LaserWeapon];
        let engines = vec![ComponentType::RaptorEngine];
        let blocks = vec![ComponentType::SteelBlock, ComponentType::Central];

        egui::Window::new("Weapons").show(&self.graphics.egui_platform.context(), |ui| {
            for weapon in weapons {
                if ui.button(weapon.to_string()).clicked() {
                    self.selected_component = Some(weapon);
                }
            }
        });
        egui::Window::new("Engines").show(&self.graphics.egui_platform.context(), |ui| {
            for engine in engines {
                if ui.button(engine.to_string()).clicked() {
                    self.selected_component = Some(engine);
                }
            }
        });
        egui::Window::new("Blocks").show(&self.graphics.egui_platform.context(), |ui| {
            for block in blocks {
                if ui.button(block.to_string()).clicked() {
                    self.selected_component = Some(block);
                }
            }
        });

        egui::Window::new("Material usage").show(&self.graphics.egui_platform.context(), |ui| {
            for (material, value) in self.spacecraft_structure.materials() {
                ui.label(format!("{}: {}", material, value));
            }
        });

        egui::Window::new("Structure data").show(&self.graphics.egui_platform.context(), |ui| {
            for tag in &mut self.spacecraft_structure.tags {
                ui.text_edit_singleline(tag);
            } 
            if ui.button("Add tag").clicked() {
                self.spacecraft_structure.tags.push(String::new());
            }

            let mut json = serde_json::to_string(&self.spacecraft_structure).unwrap();
            if ui.button("Copy JSON").clicked() {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(json.clone());
            }
            ui.collapsing("JSON", |ui| {
                ui.label(json);
            });
            
        });
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    ellipsoid::run::<SpacecraftTextures, SpacecraftBuilderApp>().await;
}
