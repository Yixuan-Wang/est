//! A router determines which engine to use for a given query.
//!
//! Routing is expected to be deterministic and fast. It must be idempotent to queries.
//! Computation-heavy operations should be implemented inside an engine, not a router.
//! Static routers are preferred, but you may use dynamic routers if necessary.
//!
//! Normally, you don't need to implement a router yourself.
//! This module provides a set of useful routers for you to use.
//! Arbitrary routers (`Arc<dyn Router + Send + Sync>`) can be combined using standard library's `HashMap` and `Vec`.
//! A fixed number of routers with definitive type can be combined using tuples.
//! An engine handle can be directly used as a router.

use crate::common::{Fail::*, Result};
use crate::engine::HandleEngine;
use crate::query::Query;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// The type of a routing result.
pub enum RouteTy {
    /// A static routing result.
    /// This guanrantees another query who *shares the same prefix* will be guaranteed to route to the same engine.
    Static,

    /// A dynamic routing result.
    /// No assumption can be made about the routing result of another query.
    Dynamic,
}

/// A routing result.
pub struct Route {
    /// The type of this routing result.
    pub ty: RouteTy,

    /// The engine to route to, represented by handle.
    pub engine: HandleEngine,
}

impl Route {
    #[inline]
    pub fn new_static(engine: HandleEngine) -> Self {
        Route {
            ty: RouteTy::Static,
            engine,
        }
    }

    #[inline]
    pub fn new_dynamic(engine: HandleEngine) -> Self {
        Route {
            ty: RouteTy::Dynamic,
            engine,
        }
    }
}

/// A [`Router`] is capable of routing a query to an engine.
pub trait Router {
    /// Try to route a query to an engine, given by a [`Route`].
    ///
    /// - If the query is successfully routed, return `Ok(Some(Route))`.
    /// - If the query cannot match any engine, return `Ok(None)`.
    /// - If an error happens, return an `Err` with [`crate::common::Fail`].
    ///
    /// If a child router cannot match any engine, the parent router should try the next child
    /// without failing instantly.
    /// However, if a child router reports an error, the parent router should propagate the error.
    ///
    /// Notice that the router **may** mutate the query.
    fn route(&self, query: &mut Query) -> Result<Option<Route>>;
}

impl Router for Arc<dyn Router + Send + Sync> {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        self.as_ref().route(query)
    }
}

/// A specialized [`RouterMap`] that stores bare engines, as leaves of a routing tree.
/// This is particularly useful and efficient for clustering engines together.
pub struct RouterMapLeaves(HashMap<String, HandleEngine>);

impl RouterMapLeaves {
    pub fn new(map: HashMap<String, HandleEngine>) -> Self {
        RouterMapLeaves(map)
    }

    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl From<HashMap<String, HandleEngine>> for RouterMapLeaves {
    fn from(map: HashMap<String, HandleEngine>) -> Self {
        RouterMapLeaves(map)
    }
}

impl std::ops::Add for RouterMapLeaves {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut map = self.0;
        map.extend(other.0);
        RouterMapLeaves(map)
    }
}

impl Deref for RouterMapLeaves {
    type Target = HashMap<String, HandleEngine>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RouterMapLeaves {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Router for RouterMapLeaves {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        let segment = query.mention.residue().first();

        let Some(segment) = segment else {
            return Ok(None);
        };

        let Some(engine) = self.0.get(*segment) else {
            return Ok(None);
        };

        query.mention.resolve_one();
        Ok(Some(Route::new_static(*engine)))
    }
}

/// A [`RouterMap`] is a map of arbitrary routers, each associated with a unique prefix.
pub type RouterMap = HashMap<String, Arc<dyn Router + Send + Sync>>;

impl Router for RouterMap {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        let segment = query.mention.residue().first().ok_or(Incomplete)?;

        let Some(router) = self.get(*segment) else {
            return Ok(None);
        };

        query.mention.resolve_one();
        router.route(query)
    }
}

/// A [`RouterVec`] is a vector of arbitrary routers, processed in order.
pub type RouterVec = Vec<Arc<dyn Router + Send + Sync>>;

impl Router for RouterVec {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        for router in self.iter() {
            match router.route(query)? {
                Some(route) => return Ok(Some(route)),
                None => (),
            }
        }

        Ok(None)
    }
}

/// A dynamic function to route a query to an engine.
/// This is useful for dynamic routing.
///
/// Note that the function must be [`Fn`], which means it can be called multiple
/// times without mutating its internal state.
pub struct RouterFn<F>(pub F)
where
    F: Fn(&mut Query) -> Result<Option<Route>>;

impl<F> Router for RouterFn<F>
where
    F: Fn(&mut Query) -> Result<Option<Route>>,
{
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        self.0(query)
    }
}

impl Router for HandleEngine {
    fn route(&self, _query: &mut Query) -> Result<Option<Route>> {
        Ok(Some(Route::new_static(*self)))
    }
}

/// A router that only routes to its inner router if no more segments
/// are left in the mention of the query.
///
/// This can be used to implement a fallback router.
pub struct RouterTerminal<R: Router>(pub R);

impl<R: Router> Router for RouterTerminal<R> {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        if query.mention.residue().is_empty() {
            self.0.route(query)
        } else {
            Ok(None)
        }
    }
}

impl<R: Router> Router for (String, R) {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        let (prefix, router) = self;
        let segment = query.mention.residue().first().ok_or(Incomplete)?;

        if prefix == *segment {
            query.mention.resolve_one();
            router.route(query)
        } else {
            Ok(None)
        }
    }
}

impl<R: Router> Router for (&str, R) {
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        let (prefix, router) = self;
        let segment = query.mention.residue().first().ok_or(Incomplete)?;

        if prefix == segment {
            query.mention.resolve_one();
            router.route(query)
        } else {
            Ok(None)
        }
    }
}

macro_rules! impl_router_for_tuple {
    ( $( $type:ident $idx:tt ),+ ) => {
        impl<$($type,)+> Router for ($($type,)+)
        where
            $($type: Router,)+
        {
            fn route(
                &self,
                query: &mut Query,
            ) -> Result<Option<Route>> {
                $(
                    match self.$idx.route(query)? {
                        val @ Some(_) => return Ok(val),
                        None => (),
                    }
                )+
                Ok(None)
            }
        }
    }
}

impl_router_for_tuple!(E0 0);
impl_router_for_tuple!(E0 0, E1 1);
impl_router_for_tuple!(E0 0, E1 1, E2 2);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5, E6 6);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5, E6 6, E7 7);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5, E6 6, E7 7, E8 8);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5, E6 6, E7 7, E8 8, E9 9);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5, E6 6, E7 7, E8 8, E9 9, E10 10);
impl_router_for_tuple!(E0 0, E1 1, E2 2, E3 3, E4 4, E5 5, E6 6, E7 7, E8 8, E9 9, E10 10, E11 11);

impl Router for () {
    fn route(&self, _query: &mut Query) -> Result<Option<Route>> {
        Ok(None)
    }
}

/// A router that asserts the inner router must route to an engine successfully.
/// Otherwise it will short-circuit the routing process and propagate the error.
pub struct RouterAssert<R: Router>(pub R);

impl<R> Router for RouterAssert<R>
where
    R: Router,
{
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        let result = self.0.route(query)?;

        if result.is_some() {
            Ok(result)
        } else {
            Err(Incomplete)
        }
    }
}

/// A router that prints debug information to the stdout in debug mode.
///
/// It will print during its entry and exit.
/// During its exit, it will print whether the routing is successful or not.
pub struct RouterDebug<R: Router>(pub R, pub &'static str);

impl<R> Router for RouterDebug<R>
where
    R: Router,
{
    fn route(&self, query: &mut Query) -> Result<Option<Route>> {
        #[cfg(debug_assertions)]
        println!(
            "{}: Enter {:?} {:?}",
            self.1,
            query.mention.resolved(),
            query.mention.residue()
        );
        let result = self.0.route(query);

        #[cfg(debug_assertions)]
        match &result {
            Ok(Some(_)) => {
                println!(
                    "{}: Ok(Some) {:?} {:?}",
                    self.1,
                    query.mention.resolved(),
                    query.mention.residue()
                );
            }
            Ok(None) => {
                println!(
                    "{}: Ok(None) {:?} {:?}",
                    self.1,
                    query.mention.resolved(),
                    query.mention.residue()
                );
            }
            Err(err) => {
                println!(
                    "{}: Err({:?}) {:?} {:?}",
                    self.1,
                    err,
                    query.mention.resolved(),
                    query.mention.residue()
                );
            }
        }
        result
    }
}
