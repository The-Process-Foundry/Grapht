//! A generic query object
//!
//! FIXME: This will move to the GQuery repo as a separate project when finished
//! THINK: Where does this project end and Grapht begin?

// The internal representation of a query
pub mod ast;

// Translation definitions for parsing and printing out queries as strings
pub mod grammars;

/* Create
  CREATE

*/

/*
    Gets a set of all the orgs and org relationships
    MATCH (:__Organization)-[r:__ParentOf|__ChildOf*0..1)-(org:__Organization)
    RETURN (n, r)
*/

// Query the root orgs
// "MATCH (org:__Organization) WHERE NOT (org)-[:__ChildOf]->(:__Organization) RETURN org"
