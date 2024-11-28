//! Router
//!
//! A router is

use crate::engine::{Engine, HandleEngine};
use crate::query::{Query, QueryMention};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Debug)]
pub enum ErrorRouting {
    Incomplete,
}

pub type Result<T> = std::result::Result<T, ErrorRouting>;

pub trait Router<E = ()> {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>>;
}

impl Router for Arc<dyn Router + Send + Sync> {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        self.as_ref().route(query)
    }
}

pub struct RouterMapLeaf(HashMap<String, HandleEngine>);

impl RouterMapLeaf {
    pub fn new(map: HashMap<String, HandleEngine>) -> Self {
        RouterMapLeaf(map)
    }

    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl std::ops::Add for RouterMapLeaf {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut map = self.0;
        map.extend(other.0);
        RouterMapLeaf(map)
    }
}

impl Deref for RouterMapLeaf {
    type Target = HashMap<String, HandleEngine>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RouterMapLeaf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Router for RouterMapLeaf {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        let segment = query
            .mention
            .residue()
            .first();

        let Some(segment) = segment else {
            return Ok(None);
        };

        let Some(engine) = self.0.get(*segment) else {
            return Ok(None);
        };
        
        query.mention.resolve_one();
        Ok(Some(*engine))
    }
}

pub struct RouterMapLayer(HashMap<String, Arc<dyn Router + Send + Sync>>);

impl RouterMapLayer {
    pub fn new(map: HashMap<String, Arc<dyn Router + Send + Sync>>) -> Self {
        RouterMapLayer(map)
    }
}

impl std::ops::Add for RouterMapLayer {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut map = self.0;
        map.extend(other.0);
        RouterMapLayer(map)
    }
}

impl Deref for RouterMapLayer {
    type Target = HashMap<String, Arc<dyn Router + Send + Sync>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RouterMapLayer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Router for RouterMapLayer {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        let segment = query
            .mention
            .residue()
            .first()
            .ok_or(ErrorRouting::Incomplete)?;

        let Some(router) = self.0.get(*segment) else {
            return Ok(None);
        };

        query.mention.resolve_one();
        router.route(query)
    }
}

pub struct RouterFunc<F>(pub F)
where
    F: Fn(&Query) -> Result<Option<HandleEngine>>;

impl<F> Router for RouterFunc<F>
where
    F: Fn(&Query) -> Result<Option<HandleEngine>>
{
    fn route(
        &self,
        query: &mut Query,
    ) -> Result<Option<HandleEngine>> {
        self.0(query)
    }
}

impl Router for HandleEngine {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        Ok(Some(*self))
    }
}

pub struct Terminal<R: Router>(pub R);

impl<R: Router> Router for Terminal<R> {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        if query.mention.residue().is_empty() {
            self.0.route(query)
        } else {
            Ok(None)
        }
    }
}

impl<R: Router> Router for (String, R) {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        let (prefix, engine) = self;
        let segment = query
            .mention
            .residue()
            .first()
            .ok_or(ErrorRouting::Incomplete)?;

        if prefix == *segment {
            query.mention.resolve_one();
            engine.route(query)
        } else {
            Ok(None)
        }
    }
}

impl<R: Router> Router for (&str, R) {
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        let (prefix, engine) = self;
        let segment = query
            .mention
            .residue()
            .first()
            .ok_or(ErrorRouting::Incomplete)?;

        if prefix == segment {
            query.mention.resolve_one();
            engine.route(query)
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
            ) -> Result<Option<HandleEngine>> {
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
    fn route(&self, _query: &mut Query) -> Result<Option<HandleEngine>> {
        Ok(None)
    }
}

pub struct Final<R: Router>(pub R);

impl<R> Router for Final<R>
where
    R: Router,
{
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        let result = self.0.route(query)?;

        if result.is_some() {
            Ok(result)
        } else {
            Err(ErrorRouting::Incomplete)
        }
    }
}

pub struct RouterDebug<R: Router>(pub R, pub &'static str);

impl<R> Router for RouterDebug<R>
where
    R: Router,
{
    fn route(&self, query: &mut Query) -> Result<Option<HandleEngine>> {
        #[cfg(debug_assertions)] println!("{}: Enter {:?} {:?}", self.1, query.mention.resolved(), query.mention.residue());
        let result = self.0.route(query);

        #[cfg(debug_assertions)] 
        match &result {
            Ok(Some(_)) => {
                println!("{}: Ok(Some) {:?} {:?}", self.1, query.mention.resolved(), query.mention.residue());
            }
            Ok(None) => {
                println!("{}: Ok(None) {:?} {:?}", self.1, query.mention.resolved(), query.mention.residue());
            }
            Err(err) => {
                println!("{}: Err({:?}) {:?} {:?}", self.1, err, query.mention.resolved(), query.mention.residue());
            }   
        }
        result
    }
}