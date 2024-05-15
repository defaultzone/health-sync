/**
 * health-sync - synchronization of server health with the game HUD.
 * Copyright (C) 2024 defaultzone.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SDPX-License-Identifier: GPL-3.0-only
 */

use std::{mem, os::raw::c_void};
use minhook::{MinHook, MH_STATUS};
use windows::Win32::{
    Foundation::{
        BOOL, HMODULE, TRUE
    },
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH}
    }
};

type CSprite2dDrawBarChart = unsafe extern "cdecl" fn(f32, f32, u16, u8, f32, i8, u8, u8, *mut c_void, *mut c_void);
static mut HOOK_ADDRESS: *mut c_void = 0x728640 as *mut _;
static mut HOOKED_FUNC: Option<CSprite2dDrawBarChart> = None;

fn install_hook() -> Result<(), MH_STATUS> {
    unsafe {
        let status = MinHook::create_hook(HOOK_ADDRESS, hooked as *mut _)?;

        HOOKED_FUNC = Some(mem::transmute(status));
        MinHook::enable_hook(HOOK_ADDRESS)?;

        Ok(())
    }
}

unsafe extern "cdecl" fn hooked(
    x: f32,
    y: f32,
    width: u16,
    height: u8,
    progress: f32,
    progress_add: i8,
    draw_percentage: u8,
    draw_black_border: u8,
    color: *mut c_void,
    add_color: *mut c_void,
) {
    if let Some(original_func) = HOOKED_FUNC {
        let mut new_progress: f32 = progress;
        
        if progress >= 10000.0 {
            new_progress = (progress - 10000.0).clamp(0.0, 160.0)
        }

        original_func(
            x,
            y,
            width,
            height,
            new_progress,
            progress_add,
            draw_percentage,
            draw_black_border,
            color,
            add_color,
        );
    }
}

#[no_mangle]
extern "stdcall" fn DllMain(instance: HMODULE, reason: u32, _reserved: *mut ()) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            match install_hook() {
                Ok(_) => println!("MinHook: CSprite2d::DrawBarChart hook is installed"),
                Err(err) => println!("MinHook: failed to install CSprite2d::DrawBarChart hook ({:?})", err),
            }

            let _ = DisableThreadLibraryCalls(instance);
        }
    } else if reason == DLL_PROCESS_DETACH {
        if let Err(err) = unsafe { MinHook::remove_hook(HOOK_ADDRESS) } {
            println!("MinHook: error removing CSprite2d::DrawBarChart hook ({:?})", err);
        }
    }

    TRUE
}
