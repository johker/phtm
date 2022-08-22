
use phtm::node::execution::PushExecutor;
use pushr::push::item::{Item, PushType};
use pushr::push::vector::IntVector;

/// Extrat integer parameter from name binding
#[allow(dead_code)]
pub fn read_int_parameter(executor: &PushExecutor, name: String) -> Option<i32> {
    match executor.push_state.name_bindings.get(&name) {
        Some(Item::Literal { push_type: pt }) => match pt {
            PushType::Int { val } => Some(*val),
            _ => None,
        },
        _ => None,
    }
}
/// Extract integer parameter from name binding
#[allow(dead_code)]
pub fn read_float_parameter(executor: &PushExecutor, name: String) -> Option<f32> {
    match executor.push_state.name_bindings.get(&name) {
        Some(Item::Literal { push_type: pt }) => match pt {
            PushType::Float { val } => Some(*val),
            _ => None,
        },
        _ => None,
    }
}

/// Extract boolean parameter from name binding
#[allow(dead_code)]
pub fn read_boolean_parameter(executor: &PushExecutor, name: String) -> Option<bool> {
    match executor.push_state.name_bindings.get(&name) {
        Some(Item::Literal { push_type: pt }) => match pt {
            PushType::Bool { val } => Some(*val),
            _ => None,
        },
        _ => None,
    }
}

/// Extracts int vector parameter from name binding
#[allow(dead_code)]
pub fn read_int_vector_parameter(executor: &PushExecutor, name: String) -> Option<Vec<i32>> {
    match executor.push_state.name_bindings.get(&name) {
        Some(Item::Literal { push_type: pt }) => match pt {
            PushType::IntVector { val } => Some(val.values.clone()),
            _ => None,
        },
        _ => None,
    }
}


/// Writes sets an integer vector to the name binding with the given name.
#[allow(dead_code)]
pub fn write_int_vector_parameter(executor: &mut PushExecutor, name: String, value: Vec<i32>)  {
    
    executor.push_state.name_bindings.insert(name, Item::intvec(IntVector::new(value)));
    }














































