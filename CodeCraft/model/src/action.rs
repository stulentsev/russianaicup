use super::*;
#[derive(Clone, Debug, Default, trans::Trans)]
pub struct Action {
    pub entity_actions: std::collections::HashMap<i32, EntityAction>,
}
