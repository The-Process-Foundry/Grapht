//! A generic query object
//!
//! FIXME: This will move to the GQuery repo as a separate project when finished
//! THINK: Where does this project end and Grapht begin?


/* Create
  CREATE

*/


/*
    Gets a set of all the orgs and org relationships
    MATCH (:__Organization)-[r:__ParentOf|__ChildOf*0..1)-(org:__Organization)
    RETURN (n, r)
*/



