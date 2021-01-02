use super::*;
#[derive(Clone, Debug, trans::Trans)]
pub enum DebugCommand {
    Add {
        data: DebugData,
    },
    Clear {
    },
    SetAutoFlush {
        enable: bool,
    },
    Flush {
    },
}

impl DebugCommand {
    pub fn draw_square(left_bottom: Vec2F32, right_top: Vec2F32, color: Color) -> Self {
        Self::Add{
            data: DebugData::Primitives {
                vertices: vec![
                    // first triangle
                    ColoredVertex{
                        world_pos: Some(Vec2F32::from_f32(left_bottom.x, left_bottom.y)),
                        screen_offset: Default::default(),
                        color
                    },
                    ColoredVertex{
                        world_pos: Some(Vec2F32::from_f32(right_top.x, left_bottom.y)),
                        screen_offset: Default::default(),
                        color
                    },
                    ColoredVertex{
                        world_pos: Some(Vec2F32::from_f32(left_bottom.x, right_top.y)),
                        screen_offset: Default::default(),
                        color
                    },

                    // second triangle
                    ColoredVertex{
                        world_pos: Some(Vec2F32::from_f32(right_top.x, right_top.y)),
                        screen_offset: Default::default(),
                        color
                    },
                    ColoredVertex{
                        world_pos: Some(Vec2F32::from_f32(left_bottom.x, right_top.y)),
                        screen_offset: Default::default(),
                        color
                    },
                    ColoredVertex{
                        world_pos: Some(Vec2F32::from_f32(right_top.x, left_bottom.y)),
                        screen_offset: Default::default(),
                        color
                    },
                ],
                primitive_type: PrimitiveType::Triangles,
            }
        }
    }

    pub fn draw_line_gradient(from: ColoredVertex, to: ColoredVertex) -> Self {
        Self::Add {
            data: DebugData::Primitives {
                vertices: vec![from, to],
                primitive_type: PrimitiveType::Lines,
            }
        }
    }
}
