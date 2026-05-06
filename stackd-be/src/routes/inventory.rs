#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "action_type")]
pub enum ActionType {
    Add,
    Update,
    Delete,
}

