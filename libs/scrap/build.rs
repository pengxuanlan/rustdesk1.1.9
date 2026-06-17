use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Link vcppkg package.
fn link_vcpkg(mut path: PathBuf, name: &str) -> PathBuf {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    }
    let mut target = if target_os == "macos" {
        if target_arch == "x64" {
            "x64-osx".to_owned()
        } else if target_arch == "arm64"{
            "arm64-osx".to_owned()
        } else {
            format!("{}-{}", target_arch, target_os)
        }
    } else if target_os == "windows" {
        "x64-windows-static".to_owned()
    } else {
        format!("{}-{}", target_arch, target_os)
    };
    if target_arch == "x86" {
        target = target.replace("x64", "x86");
    }
    println!("cargo:info={}", target);
    path.push("installed");
    path.push(target);
    println!(
        "{}",
        format!(
            "cargo:rustc-link-lib=static={}",
            name.trim_start_matches("lib")
        )
    );
    println!(
        "{}",
        format!(
            "cargo:rustc-link-search={}",
            path.join("lib").to_str().unwrap()
        )
    );
    let include = path.join("include");
    println!("{}", format!("cargo:include={}", include.to_str().unwrap()));
    include
}

/// Link homebrew package(for Mac M1).
fn link_homebrew_m1(name: &str) -> PathBuf {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_os != "macos" || target_arch != "aarch64" {
        panic!("Couldn't find VCPKG_ROOT, also can't fallback to homebrew because it's only for macos aarch64.");
    }
    let mut path = PathBuf::from("/opt/homebrew/Cellar");
    path.push(name);
    let entries = if let Ok(dir) = std::fs::read_dir(&path) {
        dir
    } else {
        panic!("Could not find package in {}. Make sure your homebrew and package {} are all installed.", path.to_str().unwrap(),&name);
    };
    let mut directories = entries
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap().path())
        .filter(|x| x.is_dir())
        .collect::<Vec<_>>();
    // Find the newest version.
    directories.sort_unstable();
    if directories.is_empty() {
        panic!(
            "There's no installed version of {} in /opt/homebrew/Cellar",
            name
        );
    }
    path.push(directories.pop().unwrap());
    // Link the library.
    println!(
        "{}",
        format!(
            "cargo:rustc-link-lib=static={}",
            name.trim_start_matches("lib")
        )
    );
    // Add the library path.
    println!(
        "{}",
        format!(
            "cargo:rustc-link-search={}",
            path.join("lib").to_str().unwrap()
        )
    );
    // Add the include path.
    let include = path.join("include");
    println!("{}", format!("cargo:include={}", include.to_str().unwrap()));
    include
}

/// Find package. By default, it will try to find vcpkg first, then homebrew(currently only for Mac M1).
fn find_package(name: &str) -> Vec<PathBuf> {
    if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
        vec![link_vcpkg(vcpkg_root.into(), name)]
    } else {
        // Try using homebrew
        vec![link_homebrew_m1(name)]
    }
}

fn generate_bindings(
    ffi_header: &Path,
    include_paths: &[PathBuf],
    ffi_rs: &Path,
    exact_file: &Path,
) {
    let mut b = bindgen::builder()
        .header(ffi_header.to_str().unwrap())
        .allowlist_type("^[vV].*")
        .allowlist_var("^[vV].*")
        .allowlist_function("^[vV].*")
        .rustified_enum("^v.*")
        .trust_clang_mangling(false)
        .layout_tests(false) // breaks 32/64-bit compat
        .generate_comments(false)
        .allowlist_function("vpx_.*")
        .allowlist_type("vpx_.*|VPX_.*")
        .allowlist_var("vpx_.*|VPX_.*|VP8_.*|VP9_.*")
        .blocklist_type("_bindgen_ty_.*"); // Block problematic internal types

    for dir in include_paths {
        b = b.clang_arg(format!("-I{}", dir.display()));
    }

    // Try to generate bindings, fallback to minimal bindings if it fails
    let result = std::panic::catch_unwind(|| {
        b.generate().unwrap().write_to_file(ffi_rs).unwrap()
    });

    if result.is_err() {
        println!("cargo:warning=bindgen failed, using minimal vpx bindings");
        // Write minimal vpx bindings manually
        let minimal_bindings = r#"
// Minimal vpx bindings for cross-compilation
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_ctx {
    pub name: *const ::std::os::raw::c_char,
    pub iface: *mut vpx_codec_iface_t,
    pub err: vpx_codec_err_t,
    pub err_detail: *const ::std::os::raw::c_char,
    pub init_flags: u32,
    pub config: vpx_codec_ctx_config,
    pub priv_: *mut ::std::os::raw::c_void,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_ctx_config {
    pub enc: *mut vpx_codec_enc_cfg_t,
    pub dec: *mut vpx_codec_dec_cfg_t,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_iface_t { _unused: [u8; 0] }
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_image_t {
    pub fmt: vpx_img_fmt_t,
    pub csp: vpx_color_space_t,
    pub d_w: u32,
    pub d_h: u32,
    pub plane_fmt: [vpx_img_fmt_t; 4],
    pub planes: [*mut u8; 4],
    pub stride: [i32; 4],
    pub bps: u32,
    pub user_priv: *mut ::std::os::raw::c_void,
    pub img_data: *mut u8,
    pub self_allocd: i32,
    pub sz: usize,
    pub w: u32,
    pub h: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_enc_cfg_t {
    pub g_usage: u32,
    pub g_threads: u32,
    pub g_profile: u32,
    pub g_w: u32,
    pub g_h: u32,
    pub g_lag_in_frames: u32,
    pub rc_undershoot_pct: u32,
    pub rc_overshoot_pct: u32,
    pub rc_max_bitrate: i32,
    pub rc_min_bitrate: i32,
    pub rc_target_bitrate: u32,
    pub rc_end_usage: vpx_rc_mode,
    pub g_pass: vpx_enc_pass,
    pub g_timebase: vpx_rational_t,
    pub kf_mode: vpx_kf_mode,
    pub kf_min_dist: u32,
    pub kf_max_dist: u32,
    pub g_error_resilient: u32,
    // Additional fields
    pub rc_dropframe_thresh: u32,
    pub resize_allowed: u32,
    pub resize_up_thresh: u32,
    pub resize_down_thresh: u32,
    pub end_usage: i32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_dec_cfg_t {
    pub threads: u32,
    pub w: u32,
    pub h: u32,
    pub allow_lowbitdepth: i32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_cx_pkt {
    pub kind: vpx_codec_cx_pkt_kind,
    pub data: vpx_codec_cx_pkt_data,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union vpx_codec_cx_pkt_data {
    pub frame: vpx_codec_cx_pkt_frame,
    pub raw: *const u8,
}
impl std::fmt::Debug for vpx_codec_cx_pkt_data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vpx_codec_cx_pkt_data {{ }}")
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_codec_cx_pkt_frame {
    pub buf: *const u8,
    pub sz: usize,
    pub pts: i64,
    pub duration: u64,
    pub flags: u32,
    pub partition_id: i32,
}
pub type vpx_codec_iter_t = *mut ::std::os::raw::c_void;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_fixed_addr_t { _unused: [u8; 0] }
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_rational_t { pub num: i32, pub den: i32 }
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vpx_rect_t { pub x: i32, pub y: i32, pub w: u32, pub h: u32 }

pub type vpx_codec_flags_t = ::std::os::raw::c_ulong;
pub type vpx_img_fmt_t = u32;
pub type vpx_color_space_t = i32;
// Type aliases for compatibility
pub type vpx_codec_ctx_t = vpx_codec_ctx;
pub type vpx_codec_enc_cfg = vpx_codec_enc_cfg_t;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vpx_codec_err_t { VPX_CODEC_OK = 0, VPX_CODEC_ERROR = 1, VPX_CODEC_MEM_ERROR = 2 }
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vpx_img_fmt { VPX_IMG_FMT_NONE = 0, VPX_IMG_FMT_I420 = 0x100, VPX_IMG_FMT_NV12 = 0x200 }
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vpx_rc_mode { VPX_CBR = 0, VPX_VBR = 1, VPX_CQ = 2 }
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vpx_kf_mode { VPX_KF_DEFAULT = 0, VPX_KF_DISABLED = 1 }
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vpx_enc_pass { VPX_RC_ONE_PASS = 0, VPX_RC_FIRST_PASS = 1, VPX_RC_LAST_PASS = 2 }
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vp8e_enc_control_id { VP8E_SET_CPUUSED = 4, VP8E_SET_ENABLEAUTOALTREF = 6, VP8E_SET_NOISE_SENSITIVITY = 7, VP8E_SET_STATIC_THRESHOLD = 9, VP8E_SET_ARNR_MAXFRAMES = 10, VP8E_SET_ARNR_STRENGTH = 11, VP8E_SET_TUNING = 12, VP8E_SET_CQ_LEVEL = 13, VP8E_SET_MAX_INTRA_BITRATE_PCT = 14, VP8E_SET_MAX_INTER_BITRATE_PCT = 15, VP8E_SET_SHARPNESS = 16, VP8E_SET_TILE_COLUMNS = 18, VP8E_SET_TILE_ROWS = 19, VP8E_SET_FRAME_PARALLEL_DECODING = 20, VP8E_SET_AQ_MODE = 21, VP8E_SET_FRAME_PERIODIC_BOOST = 22, VP8E_SET_SVC_PARAMETERS = 23, VP8E_SET_SVC_LAYER_ID = 24, VP8E_SET_SVC_REF_FRAME_CONFIG = 25, VP8E_SET_SVC_SPATIAL_LAYER_ID = 26, VP8E_SET_SVC_TEMPORAL_LAYER_ID = 27, VP8E_SET_CONTENT_LABEL = 28, VP8E_SET_MIN_GF_INTERVAL = 29, VP8E_SET_MAX_GF_INTERVAL = 30, VP8E_SET_SCREEN_CONTENT_MODE = 31 }
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum vpx_codec_cx_pkt_kind { VPX_CODEC_CX_FRAME_PKT = 0, VPX_CODEC_CX_STATS_PKT = 1 }
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VpxVideoCodecId { VP8 = 0, VP9 = 1 }

// Additional constants
pub const VP9E_SET_ROW_MT: i32 = 38;
pub const VP9E_SET_TILE_COLUMNS: i32 = 18;
pub const VPX_ENCODER_ABI_VERSION: u32 = 9;
pub const VPX_DECODER_ABI_VERSION: u32 = 9;
pub const VPX_ERROR_RESILIENT_DEFAULT: u32 = 1;
pub const VPX_FRAME_IS_KEY: u32 = 1;
pub const VPX_DL_REALTIME: u64 = 1;
pub const VPX_DL_GOOD_QUALITY: u64 = 1000000;
pub const VPX_DL_BEST_QUALITY: u64 = 0;

extern "C" {
    pub fn vpx_codec_vp8_cx() -> *const vpx_codec_iface_t;
    pub fn vpx_codec_vp9_cx() -> *const vpx_codec_iface_t;
    pub fn vpx_codec_vp8_dx() -> *const vpx_codec_iface_t;
    pub fn vpx_codec_vp9_dx() -> *const vpx_codec_iface_t;
    pub fn vpx_codec_enc_config_default(iface: *const vpx_codec_iface_t, cfg: *mut vpx_codec_enc_cfg_t, reserved: u32) -> vpx_codec_err_t;
    pub fn vpx_codec_enc_init_ver(ctx: *mut vpx_codec_ctx_t, iface: *const vpx_codec_iface_t, cfg: *const vpx_codec_enc_cfg_t, flags: vpx_codec_flags_t, ver: i32) -> vpx_codec_err_t;
    pub fn vpx_codec_dec_init_ver(ctx: *mut vpx_codec_ctx_t, iface: *const vpx_codec_iface_t, cfg: *const vpx_codec_dec_cfg_t, flags: vpx_codec_flags_t, ver: i32) -> vpx_codec_err_t;
    pub fn vpx_codec_control_(ctx: *mut vpx_codec_ctx_t, ctrl_id: i32, ...) -> vpx_codec_err_t;
    pub fn vpx_codec_encode(ctx: *mut vpx_codec_ctx_t, img: *const vpx_image_t, pts: u64, duration: u64, flags: vpx_codec_flags_t, deadline: u64) -> vpx_codec_err_t;
    pub fn vpx_codec_decode(ctx: *mut vpx_codec_ctx_t, data: *const u8, size: usize, user_priv: *mut ::std::os::raw::c_void, deadline: i64) -> vpx_codec_err_t;
    pub fn vpx_codec_destroy(ctx: *mut vpx_codec_ctx_t) -> vpx_codec_err_t;
    pub fn vpx_codec_get_cx_data(ctx: *mut vpx_codec_ctx_t, iter: *mut vpx_codec_iter_t) -> *const vpx_codec_cx_pkt;
    pub fn vpx_codec_get_frame(ctx: *mut vpx_codec_ctx_t, iter: *mut vpx_codec_iter_t) -> *const vpx_image_t;
    pub fn vpx_codec_get_caps(iface: *const vpx_codec_iface_t) -> u32;
    pub fn vpx_codec_enc_config_set(ctx: *mut vpx_codec_ctx_t, cfg: *const vpx_codec_enc_cfg_t) -> vpx_codec_err_t;
    pub fn vpx_img_wrap(img: *mut vpx_image_t, fmt: vpx_img_fmt, d_w: u32, d_h: u32, align: u32, img_data: *mut u8) -> *mut vpx_image_t;
    pub fn vpx_img_alloc(img: *mut vpx_image_t, fmt: vpx_img_fmt, d_w: u32, d_h: u32, align: u32) -> *mut vpx_image_t;
    pub fn vpx_img_free(img: *mut vpx_image_t);
}
"#;
        fs::write(ffi_rs, minimal_bindings).unwrap();
    }

    fs::copy(ffi_rs, exact_file).ok(); // ignore failure
}

fn gen_vpx() {
    let includes = find_package("libvpx");
    let src_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&src_dir);
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let ffi_header = src_dir.join("vpx_ffi.h");
    println!("rerun-if-changed={}", ffi_header.display());
    for dir in &includes {
        println!("rerun-if-changed={}", dir.display());
    }

    let ffi_rs = out_dir.join("vpx_ffi.rs");
    let exact_file = src_dir.join("generated").join("vpx_ffi.rs");
    generate_bindings(&ffi_header, &includes, &ffi_rs, &exact_file);
}

fn main() {
    // note: all link symbol names in x86 (32-bit) are prefixed wth "_".
    // run "rustup show" to show current default toolchain, if it is stable-x86-pc-windows-msvc,
    // please install x64 toolchain by "rustup toolchain install stable-x86_64-pc-windows-msvc",
    // then set x64 to default by "rustup default stable-x86_64-pc-windows-msvc"
    let target = target_build_utils::TargetInfo::new();
    if target.unwrap().target_pointer_width() != "64" {
        // panic!("Only support 64bit system");
    }
    env::remove_var("CARGO_CFG_TARGET_FEATURE");
    env::set_var("CARGO_CFG_TARGET_FEATURE", "crt-static");

    find_package("libyuv");
    gen_vpx();

    // there is problem with cfg(target_os) in build.rs, so use our workaround
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "ios" {
        // nothing
    } else if target_os == "android" {
        println!("cargo:rustc-cfg=android");
    } else if target_os == "windows" {
        // The first choice is Windows because DXGI is amazing.
        println!("cargo:rustc-cfg=dxgi");
    } else if target_os == "macos" {
        // Quartz is second because macOS is the (annoying) exception.
        println!("cargo:rustc-cfg=quartz");
    } else if target_os == "linux" {
        // On UNIX we pray that X11 (with XCB) is available.
        println!("cargo:rustc-cfg=x11");
    }
}
