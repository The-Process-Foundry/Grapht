//! Mapping various languages to the AST
//!
//! The default language is going to be GQL when the spec comes out, but since it is ill defined,
//! I'm going to pull
//! GQL: https://s3.amazonaws.com/artifacts.opencypher.org/website/materials/sql-pg-2018-0046r3-GQL-Scope-and-Features.pdf

// The official GQL language implementation
// pub mod gql;

// An implementation of OpenCypher
pub mod cypher;
