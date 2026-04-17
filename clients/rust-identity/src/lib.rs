extern crate self as borsh;

pub use ::borsh_dep::*;

pub mod maybestd {
    pub mod io {
        pub use ::borsh_dep::io::*;
    }
}

mod generated;

pub use generated::programs::MPL_AGENT_IDENTITY_ID as ID;
pub use generated::*;
