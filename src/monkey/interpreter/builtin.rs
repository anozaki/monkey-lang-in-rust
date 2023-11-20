use std::fmt::format;
use crate::monkey::interpreter::{NULL, Object};
use crate::monkey::Result;

pub fn str_len(objs: Vec<Object>) -> Result<Object> {
    let Some(obj) = objs.first() else {
        return Ok(Object::Error(format!("len(): Invalid argument")))
    };

    Ok(match obj {
        Object::String(str) => Object::Int(str.len() as isize),
        Object::Array(arr) => Object::Int(arr.len() as isize),
        _ => Object::Error(format!("len(): Invalid argument: {:?}", obj)),
    })
}

pub fn first(objs: Vec<Object>) -> Result<Object> {
    let Some(obj) = objs.first() else {
        return Ok(Object::Error(format!("first(): Invalid argument")))
    };

    Ok(match obj {
        Object::Array(arr) => match arr.first() {
            None => NULL,
            Some(val) => val.clone()
        },
        _ => Object::Error(format!("first(): Invalid argument: {:?}", obj)),
    })
}

pub fn last(objs: Vec<Object>) -> Result<Object> {
    let Some(obj) = objs.first() else {
        return Ok(Object::Error(format!("last(): Invalid argument")))
    };

    Ok(match obj {
        Object::Array(arr) => match arr.last() {
            None => NULL,
            Some(val) => val.clone()
        },
        _ => Object::Error(format!("last(): Invalid argument: {:?}", obj)),
    })
}

pub fn push(objs: Vec<Object>) -> Result<Object> {
    if objs.len() != 2 {
        return Ok(Object::Error(format!("push(): Invalid argument")))
    }

    let target = &objs[0];
    let obj = &objs[1];

    Ok(match target {
        Object::Array(arr) => {
            let mut result = arr.clone();
            result.push(obj.clone());

            Object::Array(result)
        },
        _ => Object::Error(format!("push(): Invalid argument: {:?}", obj)),
    })
}

pub fn rest(objs: Vec<Object>) -> Result<Object> {
    let Some(obj) = objs.first() else {
        return Ok(Object::Error(format!("rest(): Invalid argument")))
    };

    Ok(match obj {
        Object::Array(arr) => {
            if let Some((last, elements)) = arr.split_last() {
                Object::Array(Vec::from(elements))
            } else {
                Object::Array(vec![])
            }
        },
        _ => Object::Error(format!("rest(): Invalid argument: {:?}", obj)),
    })
}

pub fn put(objs: Vec<Object>) -> Result<Object> {
    if objs.len() != 3 {
        return Ok(Object::Error(format!("put(): Invalid argument")))
    }

    let target = &objs[0];
    let key = &objs[1];
    let val = &objs[2];

    Ok(match target {
        Object::Hash(map) => {
            let mut result = map.clone();
            result.insert(key.clone(), val.clone());

            Object::Hash(result)
        },
        _ => Object::Error(format!("put(): Invalid argument")),
    })
}
