extern crate libc;

use std::ffi::{CStr, CString};
use std::io;
use std::mem;
use std::path::Path;
use std::ptr;

#[link(name = "nereon")]
extern "C" {
    fn nereon_ctx_init(
        ctx: *mut Ctx,
        cfg_path: *const libc::c_char,
        meta_path: *const libc::c_char,
    ) -> libc::c_int;
    fn nereon_ctx_finalize(ctx: *mut Ctx) -> ();
}

const CFG_MAX_NAME: usize = 64;
const CFG_MAX_LONG_SWITCH: usize = 32;
const CFG_MAX_SHORT_DESC: usize = 32;
const CFG_MAX_LONG_DESC: usize = 128;
const CFG_MAX_ENV_NAME: usize = 64;
const CFG_MAX_KEY_NAME: usize = 128;
const CFG_MAX_ERR_MSG: usize = 1024;

#[repr(C)]
struct Ctx {
    meta: *mut libc::c_void,
    meta_count: libc::c_int,
    cfg: *const libc::c_void,
}

#[repr(C)]
struct Cfg {
    cfg_key: [libc::c_char; CFG_MAX_KEY_NAME],
    cfg_type: libc::c_int,

    childs: *const libc::c_void,
    next: *const libc::c_void,

    cfg_data: f64,
}

#[repr(C)]
struct Meta {
    cfg_name: [libc::c_char; CFG_MAX_NAME],
    cfg_type: libc::c_int,

    helper: bool,

    sw_short: [libc::c_char; 2],
    sw_long: [libc::c_char; CFG_MAX_LONG_SWITCH],

    desc_short: [libc::c_char; CFG_MAX_SHORT_DESC],
    desc_long: [libc::c_char; CFG_MAX_LONG_DESC],

    cfg_env: [libc::c_char; CFG_MAX_ENV_NAME],
    cfg_key: [libc::c_char; CFG_MAX_KEY_NAME],

    cfg_data: f64,
}

#[repr(C)]
struct Data {
    d: [libc::c_char; 8],
}

struct PathOrNull {
    cstr: CString,
    ptr: *const libc::c_char,
}

impl PathOrNull {
    fn new(path: Option<&Path>) -> PathOrNull {
        match path {
            Some(p) => {
                let cstr = CString::new(p.to_str().unwrap()).unwrap();
                PathOrNull {
                    ptr: cstr.as_ptr(),
                    cstr: cstr,
                }
            }
            None => PathOrNull {
                cstr: CString::new("").unwrap(),
                ptr: ptr::null(),
            },
        }
    }
}

pub fn nereon(
    cfg: Option<&Path>,
    meta: Option<&Path>,
) -> io::Result<(Option<super::Cfg>, Vec<super::Meta>)> {
    let mut ctx = Ctx {
        meta: ptr::null_mut(),
        meta_count: 0,
        cfg: ptr::null(),
    };

    let cfg = PathOrNull::new(cfg);

    unsafe { libc::puts(cfg.ptr) };

    if unsafe { nereon_ctx_init(&mut ctx, cfg.ptr, PathOrNull::new(meta).ptr) } == -1 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get nereon configuration.",
        ));
    }

    let mut cfg = None;

    // top object is defunct - root object is first child
    if !ctx.cfg.is_null() {
        let defunct = unsafe { ptr::read(ctx.cfg as *const Cfg) };
        if !defunct.childs.is_null() {
            let cfg_rec: Cfg = unsafe { ptr::read(defunct.childs as *const Cfg) };
            cfg = Some(get_cfg(&cfg_rec));
        }
    }

    unsafe { nereon_ctx_finalize(&mut ctx) };

    Ok((cfg, vec![]))
}

fn get_cfg(cfg_rec: &Cfg) -> super::Cfg {
    let key = unsafe { CStr::from_ptr(cfg_rec.cfg_key.as_ptr()) }
        .to_str()
        .unwrap()
        .to_owned();
    let data = match cfg_rec.cfg_type {
        0 => {
            let mut i: i64 = 0;
            unsafe {
                ptr::copy(
                    &cfg_rec.cfg_data as *const _ as *const u8,
                    &i as *const _ as *mut u8,
                    8,
                );
            }
            super::CfgData::Int(i)
        }
        2 => {
            let mut s: *const libc::c_char;
            unsafe {
                s = mem::uninitialized();
                ptr::copy(
                    &cfg_rec.cfg_data as *const _ as *const u8,
                    &s as *const _ as *mut u8,
                    mem::size_of::<*const libc::c_void>(),
                );
            }
            super::CfgData::String(unsafe { CStr::from_ptr(s) }.to_str().unwrap().to_owned())
        }
        3 => super::CfgData::Array(get_cfg_childs(cfg_rec)),
        6 => super::CfgData::Object(get_cfg_childs(cfg_rec)),
        n => panic!("aaarggghhh {}", n),
    };

    super::Cfg {
        key: key,
        data: data,
    }
}

fn get_cfg_childs(cfg_rec: &Cfg) -> Vec<super::Cfg> {
    let mut cfgs = vec![];
    let mut addr = cfg_rec.childs;

    while !addr.is_null() {
        let cfg_rec: Cfg = unsafe { ptr::read(addr as *const Cfg) };
        cfgs.push(get_cfg(&cfg_rec));
        addr = cfg_rec.next;
    }
    cfgs
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    //    use super::*;

    #[test]
    fn nereon() {
        //        match super::nereon(None, None) {
        //            Ok((c, m)) => assert_eq!((c.len(), m.len()), (0,0)),
        //            _ => panic!("nereon not behaving")
        //        }
        match super::nereon(
            Some(Path::new("./testdata/cfg.hcl")),
            Some(Path::new("./testdata/cmdline.hcl")),
        ) {
            Ok((Some(c), m)) => {
                println!("{:?}", c);
                assert_eq!(m.len(), 0);
            }
            _ => panic!("nereon with cfg not behaving"),
        }
    }
}
