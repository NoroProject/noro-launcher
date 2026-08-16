pub mod agent_auth;
pub mod discord;
pub mod middleware;
pub mod oauth2_provider;
pub mod passkeys;
pub mod webauthn;
pub mod yggdrasil;

pub use agent_auth::{generate_agent_secret, hash_agent_secret, AgentAuth};
pub use middleware::{hash_admin_token, AdminAuth, AuthUser};
