// Re-export order DTOs from the dto module  
pub use crate::dto::order::*;

// Note: Some legacy order structures may exist here temporarily for backward compatibility
// Consider migrating all order-related logic to use the dto module